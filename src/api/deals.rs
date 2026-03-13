use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::Db;
use crate::db::PriceRepository;

#[derive(Debug, Deserialize)]
pub struct DealsQuery {
    pub limit: Option<u32>,
    pub route: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DealResponse {
    pub route: String,
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
    pub price_usd: f64,
    pub baseline_mean: f64,
    pub pct_below: f64,
    pub severity: String,
    pub source: String,
    pub booking_url: Option<String>,
    pub alerted_at: String,
}

pub async fn list_deals(
    Query(params): Query<DealsQuery>,
    State(db): State<Db>,
) -> Json<Vec<DealResponse>> {
    let limit = params.limit.unwrap_or(50);

    let alerts = db.recent_alerts(limit).await.unwrap_or_default();

    let filtered: Vec<DealResponse> = alerts
        .into_iter()
        .filter(|a| {
            if let Some(route_filter) = &params.route {
                let route_key = format!("{}-{}", a.origin, a.destination);
                return route_key.eq_ignore_ascii_case(route_filter);
            }
            true
        })
        .map(|a| DealResponse {
            route: format!("{}-{}", a.origin, a.destination),
            origin: a.origin,
            destination: a.destination,
            departure_date: a.departure_date.to_string(),
            price_usd: a.price_usd,
            baseline_mean: a.baseline_mean,
            pct_below: a.pct_below,
            severity: a.severity,
            source: a.source,
            booking_url: a.booking_url,
            alerted_at: a.alerted_at.to_rfc3339(),
        })
        .collect();

    Json(filtered)
}
