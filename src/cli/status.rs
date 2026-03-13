use anyhow::Result;
use chrono::Utc;

use crate::baseline::LEARNING_DAYS_REQUIRED;
use crate::config::default_routes;
use crate::db::PriceRepository;
use crate::db::SqliteRepo;

pub async fn run(db: &SqliteRepo) -> Result<()> {
    let routes = default_routes();
    println!(
        "{:<12} {:<12} {:<10} {:<16} {:<20} {:<14}",
        "Route", "Phase", "Days Data", "Last Fetched", "Best Weekend $", "Baseline Mean"
    );
    println!("{}", "-".repeat(88));

    for route in &routes {
        for dest in &route.destinations {
            let days = db.days_of_data_for_route(&route.origin, dest).await.unwrap_or(0);
            let phase = if days as i64 >= LEARNING_DAYS_REQUIRED { "ACTIVE" } else { "LEARNING" };
            let baseline = db.baseline_for_route(&route.origin, dest).await.ok().flatten();
            let prices = db.prices_for_route_by_days(&route.origin, dest, 90, None).await.unwrap_or_default();
            let best_price = prices.iter().map(|p| p.price_usd).reduce(f64::min);

            let last_fetched = prices
                .first()
                .map(|p| {
                    let hours_ago = (Utc::now() - p.fetched_at).num_hours();
                    if hours_ago < 1 {
                        "<1h ago".to_string()
                    } else {
                        format!("{}h ago", hours_ago)
                    }
                })
                .unwrap_or_else(|| "never".to_string());

            let best_str = best_price
                .map(|p| format!("${:.0}", p))
                .unwrap_or_else(|| "—".to_string());

            let mean_str = match &baseline {
                Some(b) if phase == "ACTIVE" => format!("${:.0}", b.mean_usd),
                _ => "—".to_string(),
            };

            let phase_display = if phase == "LEARNING" {
                format!("LEARNING (day {} of {})", days, LEARNING_DAYS_REQUIRED)
            } else {
                "ACTIVE".to_string()
            };

            println!(
                "{:<12} {:<22} {:<10} {:<16} {:<20} {:<14}",
                format!("{}→{}", route.origin, dest),
                phase_display,
                days,
                last_fetched,
                best_str,
                mean_str,
            );
        }
    }
    Ok(())
}
