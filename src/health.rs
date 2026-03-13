use chrono::{Duration, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub hours_since_last_run: Option<f64>,
}

pub async fn get_health() -> HealthStatus {
    let last_run = crate::scheduler::read_last_run().await;

    match last_run {
        None => HealthStatus {
            status: "no_runs_yet".to_string(),
            last_run: None,
            next_run: None,
            hours_since_last_run: None,
        },
        Some(last) => {
            let now = Utc::now();
            let hours_since = (now - last).num_minutes() as f64 / 60.0;
            let next = last + Duration::hours(12);
            let status = if hours_since < 13.0 { "ok" } else { "stale" };

            HealthStatus {
                status: status.to_string(),
                last_run: Some(last.to_rfc3339()),
                next_run: Some(next.to_rfc3339()),
                hours_since_last_run: Some((hours_since * 100.0).round() / 100.0),
            }
        }
    }
}
