use anyhow::Result;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::config::DedupConfig;
use crate::db::PriceRepository;
use crate::detector::Deal;

/// Compute a deduplication fingerprint for a deal.
/// Uses: origin + destination + departure_date + price_bucket
pub fn compute_fingerprint(deal: &Deal, bucket_size: f64) -> String {
    let bucket = (deal.price_usd / bucket_size).floor() * bucket_size;
    let raw = format!(
        "{}|{}|{}|{}",
        deal.origin, deal.destination, deal.departure_date, bucket as u64
    );
    let hash = Sha256::digest(raw.as_bytes());
    hex::encode(hash)
}

/// Check whether we should alert for this deal.
/// Returns true if the deal is new (not yet alerted for this price tier).
pub async fn should_alert(
    deal: &Deal,
    repo: &dyn PriceRepository,
    cfg: &DedupConfig,
) -> Result<bool> {
    let fingerprint = compute_fingerprint(deal, cfg.bucket_size);

    // If exact fingerprint already exists, suppress
    if repo.alert_exists(&fingerprint).await? {
        debug!(
            origin = deal.origin,
            destination = deal.destination,
            departure_date = %deal.departure_date,
            fingerprint,
            "Alert suppressed: fingerprint already exists"
        );
        return Ok(false);
    }

    // Check re-alert condition: only re-alert if price dropped >realer_threshold below last alerted
    if let Some(last) = repo
        .latest_alert_for_route_date(&deal.origin, &deal.destination, deal.departure_date)
        .await?
    {
        let realer_threshold = last.price_usd * (1.0 - cfg.realer_threshold);
        if deal.price_usd >= realer_threshold {
            debug!(
                origin = deal.origin,
                destination = deal.destination,
                last_price = last.price_usd,
                new_price = deal.price_usd,
                "Alert suppressed: insufficient price drop since last alert"
            );
            return Ok(false);
        }
    }

    Ok(true)
}

/// Filter a list of deals down to only those that should generate alerts.
/// Returns (deal, fingerprint) pairs ready to be sent and recorded.
pub async fn filter_deals(
    deals: Vec<Deal>,
    repo: &dyn PriceRepository,
    cfg: &DedupConfig,
) -> Result<Vec<(Deal, String)>> {
    let mut to_alert = Vec::new();
    for deal in deals {
        if should_alert(&deal, repo, cfg).await? {
            let fp = compute_fingerprint(&deal, cfg.bucket_size);
            to_alert.push((deal, fp));
        }
    }
    Ok(to_alert)
}
