use serde::{Deserialize, Serialize};

use crate::baseline::RouteStatus;
use crate::config::DetectionConfig;
use crate::fetcher::types::{PriceResult, Source};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DealSeverity {
    GoodDeal,
    ExceptionalDeal,
}

impl std::fmt::Display for DealSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DealSeverity::GoodDeal => write!(f, "good_deal"),
            DealSeverity::ExceptionalDeal => write!(f, "exceptional_deal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub origin: String,
    pub destination: String,
    pub departure_date: chrono::NaiveDate,
    pub return_date: chrono::NaiveDate,
    pub price_usd: f64,
    pub baseline_mean: f64,
    pub baseline_p25: f64,
    pub pct_below_mean: f64,
    pub stddev_multiplier: f64,
    pub severity: DealSeverity,
    pub source: Source,
    pub booking_url: Option<String>,
}

/// Evaluate a batch of fresh prices against their route baselines.
/// Skips routes still in the learning phase.
pub fn evaluate(prices: &[PriceResult], statuses: &[RouteStatus], cfg: &DetectionConfig) -> Vec<Deal> {
    let mut deals = Vec::new();

    for price in prices {
        let key = (&price.origin, &price.destination);

        let status = statuses
            .iter()
            .find(|s| s.origin() == key.0 && s.destination() == key.1);

        let Some(RouteStatus::Ready { baseline, .. }) = status else {
            continue; // still learning — skip
        };

        if let Some(deal) = is_deal(price, baseline, cfg) {
            deals.push(deal);
        }
    }

    deals
}

fn is_deal(
    price: &PriceResult,
    baseline: &crate::db::types::Baseline,
    cfg: &DetectionConfig,
) -> Option<Deal> {
    let stddev_threshold = baseline.mean_usd - cfg.stddev_multiplier * baseline.stddev_usd;
    let p25_threshold = baseline.p25_usd * cfg.p25_factor;

    let below_stddev = price.price_usd < stddev_threshold;
    let below_p25 = price.price_usd < p25_threshold;

    if !below_stddev && !below_p25 {
        return None;
    }

    let severity = if below_stddev && below_p25 {
        DealSeverity::ExceptionalDeal
    } else {
        DealSeverity::GoodDeal
    };

    let pct_below_mean = if baseline.mean_usd > 0.0 {
        (baseline.mean_usd - price.price_usd) / baseline.mean_usd * 100.0
    } else {
        0.0
    };

    let stddev_mult = if baseline.stddev_usd > 0.0 {
        (baseline.mean_usd - price.price_usd) / baseline.stddev_usd
    } else {
        0.0
    };

    Some(Deal {
        origin: price.origin.clone(),
        destination: price.destination.clone(),
        departure_date: price.departure_date,
        return_date: price.return_date,
        price_usd: price.price_usd,
        baseline_mean: baseline.mean_usd,
        baseline_p25: baseline.p25_usd,
        pct_below_mean: (pct_below_mean * 100.0).round() / 100.0,
        stddev_multiplier: (stddev_mult * 100.0).round() / 100.0,
        severity,
        source: price.source.clone(),
        booking_url: price.booking_url.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::Baseline;
    use chrono::Utc;

    fn make_baseline(mean: f64, stddev: f64, p25: f64) -> Baseline {
        Baseline {
            id: None,
            origin: "LAX".into(),
            destination: "NRT".into(),
            mean_usd: mean,
            median_usd: mean,
            p10_usd: mean - stddev * 2.0,
            p25_usd: p25,
            stddev_usd: stddev,
            sample_count: 100,
            days_of_data: 30,
            computed_at: Utc::now(),
        }
    }

    fn make_price(price: f64) -> PriceResult {
        use chrono::NaiveDate;
        PriceResult {
            origin: "LAX".into(),
            destination: "NRT".into(),
            departure_date: NaiveDate::from_ymd_opt(2026, 4, 3).unwrap(),
            return_date: NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
            price_usd: price,
            source: Source::GoogleFlights,
            fetched_at: Utc::now(),
            is_scraped: false,
            booking_url: None,
        }
    }

    fn cfg() -> DetectionConfig {
        DetectionConfig { stddev_multiplier: 1.5, p25_factor: 0.85 }
    }

    #[test]
    fn good_deal_stddev_only() {
        // mean=900, stddev=100 -> threshold=750. Price=700 -> below stddev but not p25
        let b = make_baseline(900.0, 100.0, 850.0); // p25=850, p25*0.85=722.5, price=700 < 722.5 too
        // Actually price=740: below stddev (750) but above p25 threshold (722.5)
        let b2 = make_baseline(900.0, 100.0, 900.0); // p25=900, p25*0.85=765
        let p = make_price(740.0);
        let deal = is_deal(&p, &b2, &cfg());
        assert!(deal.is_some());
        assert_eq!(deal.unwrap().severity, DealSeverity::GoodDeal);
    }

    #[test]
    fn exceptional_deal_both_thresholds() {
        // mean=900, stddev=100 -> stddev_threshold=750; p25=800, p25*0.85=680
        let b = make_baseline(900.0, 100.0, 800.0);
        let p = make_price(650.0); // below both
        let deal = is_deal(&p, &b, &cfg());
        assert!(deal.is_some());
        assert_eq!(deal.unwrap().severity, DealSeverity::ExceptionalDeal);
    }

    #[test]
    fn no_deal_normal_price() {
        let b = make_baseline(900.0, 100.0, 800.0);
        let p = make_price(870.0);
        assert!(is_deal(&p, &b, &cfg()).is_none());
    }
}
