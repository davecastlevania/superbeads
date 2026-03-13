use std::sync::Arc;

use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use flight_tracker::{
    baseline::compute_baseline,
    cli,
    config::Config,
    dates::weekend_dates,
    db::{PriceRepository, SqliteRepo},
    dedup::filter_deals,
    detector::evaluate,
    fetcher::Fetcher,
    notifier::Notifier,
    scheduler::{record_last_run, Scheduler},
};

#[derive(Parser)]
#[command(name = "tracker", about = "Flight Deal Tracker")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show status for all monitored routes
    Status,
    /// Show price history for a route (e.g. LAX-NRT)
    History {
        route: String,
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// Manually trigger a full fetch cycle (no alerts sent)
    Backfill {
        #[arg(long)]
        route: Option<String>,
    },
    /// Send a test alert to Discord without a real check cycle
    TestAlert {
        #[arg(long)]
        route: String,
        #[arg(long)]
        price: f64,
        #[arg(long)]
        baseline: f64,
    },
    /// Run the scheduler (default)
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::load()?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.log_level));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .init();

    let db = Arc::new(SqliteRepo::new(&cfg.db_path).await?);
    let args = Cli::parse();

    match args.command.unwrap_or(Command::Run) {
        Command::Status => {
            cli::status::run(&db).await?;
        }
        Command::History { route, days } => {
            cli::history::run(&db, &route, days).await?;
        }
        Command::Backfill { route } => {
            cli::backfill::run(&db, route.as_deref(), &cfg).await?;
        }
        Command::TestAlert { route, price, baseline } => {
            run_test_alert(&route, price, baseline, &cfg).await?;
        }
        Command::Run => {
            run_scheduler(db, cfg).await?;
        }
    }

    Ok(())
}

async fn run_test_alert(route: &str, price: f64, baseline_mean: f64, cfg: &Config) -> Result<()> {
    let parts: Vec<&str> = route.splitn(2, '-').collect();
    anyhow::ensure!(parts.len() == 2, "Route must be ORIGIN-DEST");

    let notifier = Notifier::from_env()?;
    let db = SqliteRepo::new(&cfg.db_path).await?;

    use chrono::{NaiveDate, Utc};
    use flight_tracker::detector::{Deal, DealSeverity};
    use flight_tracker::fetcher::types::Source;

    let today = Local::now().date_naive();
    let deal = Deal {
        origin: parts[0].to_string(),
        destination: parts[1].to_string(),
        departure_date: today + chrono::Duration::days(7),
        return_date: today + chrono::Duration::days(10),
        price_usd: price,
        baseline_mean,
        baseline_p25: baseline_mean * 0.9,
        pct_below_mean: (baseline_mean - price) / baseline_mean * 100.0,
        stddev_multiplier: 1.8,
        severity: DealSeverity::ExceptionalDeal,
        source: Source::GoogleFlights,
        booking_url: None,
    };

    notifier.send_alert(&deal, "test-fingerprint", &db, true).await?;
    println!("Test alert sent successfully.");
    Ok(())
}

async fn run_scheduler(db: Arc<SqliteRepo>, cfg: Config) -> Result<()> {
    info!(event = "startup", "Flight Deal Tracker starting");

    // Run an immediate check if last run was >12h ago (or never)
    let last = flight_tracker::scheduler::read_last_run().await;
    let should_run_now = last
        .map(|t| (chrono::Utc::now() - t).num_hours() >= 12)
        .unwrap_or(true);

    if should_run_now {
        info!("Running immediate check on startup");
        run_check(db.clone(), cfg.clone()).await;
    }

    let mut sched = Scheduler::new().await?;
    let db2 = db.clone();
    let cfg2 = cfg.clone();

    sched
        .add_check_job(move || {
            let db = db2.clone();
            let cfg = cfg2.clone();
            async move {
                run_check(db, cfg).await;
            }
        })
        .await?;

    sched.start().await?;

    info!("Scheduler running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    sched.shutdown().await?;
    Ok(())
}

async fn run_check(db: Arc<SqliteRepo>, cfg: Config) {
    info!(event = "run_start", routes = cfg.routes.len());

    let fetcher = Fetcher::new();
    let today = Local::now().date_naive();
    let dates = weekend_dates(today, cfg.window_days);

    // Fetch prices for all routes
    let mut all_prices = Vec::new();
    for route in &cfg.routes {
        match tokio::time::timeout(
            std::time::Duration::from_secs(120),
            fetcher.fetch_route(&route.origin, &route.destinations, &dates),
        )
        .await
        {
            Ok(prices) => all_prices.extend(prices),
            Err(_) => error!(origin = route.origin, "Fetch timed out after 120s"),
        }
    }

    // Store prices
    if let Err(e) = db.insert_prices(&all_prices).await {
        error!(error = %e, "Failed to insert prices");
    }

    // Recompute baselines and detect deals
    let mut statuses = Vec::new();
    for route in &cfg.routes {
        for dest in &route.destinations {
            match compute_baseline(db.as_ref(), &route.origin, dest).await {
                Ok(s) => statuses.push(s),
                Err(e) => error!(origin = route.origin, destination = dest, error = %e, "Baseline error"),
            }
        }
    }

    let deals = evaluate(&all_prices, &statuses, &cfg.detection);

    if deals.is_empty() {
        info!(event = "run_complete", deals_found = 0);
        record_last_run().await.ok();
        return;
    }

    // Filter duplicates and send alerts
    match filter_deals(deals, db.as_ref(), &cfg.dedup).await {
        Ok(to_alert) => {
            if let Ok(notifier) = Notifier::from_env() {
                for (deal, fp) in &to_alert {
                    if let Err(e) = notifier.send_alert(deal, fp, db.as_ref(), false).await {
                        error!(error = %e, "Failed to send Discord alert");
                    }
                }
            } else {
                info!(deals = to_alert.len(), "Discord not configured; skipping alerts");
            }
            info!(event = "run_complete", deals_alerted = to_alert.len());
        }
        Err(e) => error!(error = %e, "Dedup filter failed"),
    }

    record_last_run().await.ok();
}
