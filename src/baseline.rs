use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};

use crate::db::{types::Baseline, PriceRepository};

pub const LEARNING_DAYS_REQUIRED: i64 = 14;

#[derive(Debug, Clone)]
pub enum RouteStatus {
    Learning {
        origin: String,
        destination: String,
        days_collected: i64,
        days_remaining: i64,
    },
    Ready {
        origin: String,
        destination: String,
        baseline: Baseline,
    },
}

impl RouteStatus {
    pub fn origin(&self) -> &str {
        match self {
            RouteStatus::Learning { origin, .. } | RouteStatus::Ready { origin, .. } => origin,
        }
    }

    pub fn destination(&self) -> &str {
        match self {
            RouteStatus::Learning { destination, .. }
            | RouteStatus::Ready { destination, .. } => destination,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, RouteStatus::Ready { .. })
    }

    pub fn baseline(&self) -> Option<&Baseline> {
        match self {
            RouteStatus::Ready { baseline, .. } => Some(baseline),
            _ => None,
        }
    }
}

/// Compute (or recompute) the baseline for one route.
pub async fn compute_baseline(
    repo: &dyn PriceRepository,
    origin: &str,
    destination: &str,
) -> Result<RouteStatus> {
    let days_of_data = repo.days_of_data_for_route(origin, destination).await? as i64;

    if days_of_data < LEARNING_DAYS_REQUIRED {
        let days_remaining = LEARNING_DAYS_REQUIRED - days_of_data;
        info!(
            event = "learning",
            origin,
            destination,
            days_collected = days_of_data,
            days_remaining,
            "Route still in learning phase"
        );
        return Ok(RouteStatus::Learning {
            origin: origin.to_string(),
            destination: destination.to_string(),
            days_collected: days_of_data,
            days_remaining,
        });
    }

    let epoch = chrono::DateTime::from_timestamp(0, 0).unwrap();
    let prices = repo.prices_for_route(origin, destination, epoch).await?;

    if prices.is_empty() {
        warn!(origin, destination, "No prices despite days_of_data >= 14");
        return Ok(RouteStatus::Learning {
            origin: origin.to_string(),
            destination: destination.to_string(),
            days_collected: 0,
            days_remaining: LEARNING_DAYS_REQUIRED,
        });
    }

    let mut values: Vec<f64> = prices.iter().map(|p| p.price_usd).collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    let median = percentile(&values, 0.50);
    let p10 = percentile(&values, 0.10);
    let p25 = percentile(&values, 0.25);

    let baseline = Baseline {
        id: None,
        origin: origin.to_string(),
        destination: destination.to_string(),
        mean_usd: mean,
        median_usd: median,
        p10_usd: p10,
        p25_usd: p25,
        stddev_usd: stddev,
        sample_count: values.len() as i64,
        days_of_data,
        computed_at: Utc::now(),
    };

    repo.upsert_baseline(&baseline).await?;

    info!(
        event = "baseline_computed",
        origin,
        destination,
        mean_usd = mean,
        p25_usd = p25,
        stddev_usd = stddev,
        sample_count = values.len(),
        days_of_data,
    );

    Ok(RouteStatus::Ready {
        origin: origin.to_string(),
        destination: destination.to_string(),
        baseline,
    })
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
