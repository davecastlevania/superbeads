use anyhow::Result;
use chrono::Local;
use tracing::info;

use crate::config::{default_routes, Config};
use crate::dates::weekend_dates;
use crate::db::{PriceRepository, SqliteRepo};
use crate::fetcher::Fetcher;

/// Manually trigger a full fetch cycle without sending alerts.
pub async fn run(db: &SqliteRepo, route_filter: Option<&str>, cfg: &Config) -> Result<()> {
    let routes = default_routes();
    let fetcher = Fetcher::new();
    let today = Local::now().date_naive();
    let dates = weekend_dates(today, cfg.window_days);

    for route in &routes {
        if let Some(filter) = route_filter {
            let key = route.origin.to_ascii_uppercase();
            if !key.eq_ignore_ascii_case(filter.split('-').next().unwrap_or("")) {
                continue;
            }
        }

        info!(
            event = "backfill_route",
            origin = route.origin,
            dates = dates.len(),
        );

        let prices = fetcher.fetch_route(&route.origin, &route.destinations, &dates).await;
        let count = prices.len();
        db.insert_prices(&prices).await?;
        println!("  {} → {:?}: {} prices fetched", route.origin, route.destinations, count);
    }

    println!("\nBackfill complete. No alerts were sent.");
    Ok(())
}
