use anyhow::Result;

use crate::db::PriceRepository;
use crate::db::SqliteRepo;

pub async fn run(db: &SqliteRepo, route: &str, days: u32) -> Result<()> {
    let parts: Vec<&str> = route.splitn(2, '-').collect();
    if parts.len() != 2 {
        anyhow::bail!("Route must be in format ORIGIN-DEST, e.g. LAX-NRT");
    }
    let (origin, dest) = (parts[0], parts[1]);

    let prices = db.prices_for_route_by_days(origin, dest, days, None).await?;

    if prices.is_empty() {
        println!("No price data for {} in the last {} days.", route, days);
        return Ok(());
    }

    println!(
        "{:<26} {:<14} {:<12} {:<16} {:<10}",
        "Fetched At", "Departure", "Price (USD)", "Source", "Scraped?"
    );
    println!("{}", "-".repeat(80));

    for p in &prices {
        println!(
            "{:<26} {:<14} ${:<11.2} {:<16} {}",
            p.fetched_at.format("%Y-%m-%d %H:%M UTC"),
            p.departure_date.to_string(),
            p.price_usd,
            p.source,
            if p.is_scraped { "yes" } else { "no" },
        );
    }

    println!("\nTotal: {} records", prices.len());
    Ok(())
}
