use axum::{extract::State, Json};
use serde::Serialize;

use crate::api::Db;
use crate::baseline::LEARNING_DAYS_REQUIRED;
use crate::config::default_routes;
use crate::db::PriceRepository;

#[derive(Debug, Serialize)]
pub struct RouteResponse {
    pub route: String,
    pub origin: String,
    pub destination: String,
    pub phase: String,
    pub days_of_data: u32,
    pub last_fetched_at: Option<String>,
    pub best_weekend_price_usd: Option<f64>,
    pub baseline_mean_usd: Option<f64>,
    pub baseline_p25_usd: Option<f64>,
}

pub async fn list_routes(State(db): State<Db>) -> Json<Vec<RouteResponse>> {
    let routes = default_routes();
    let mut result = Vec::new();

    for route in &routes {
        for dest in &route.destinations {
            let days = db
                .days_of_data_for_route(&route.origin, dest)
                .await
                .unwrap_or(0);

            let phase = if days as i64 >= LEARNING_DAYS_REQUIRED {
                "active"
            } else {
                "learning"
            };

            let baseline = db.baseline_for_route(&route.origin, dest).await.ok().flatten();
            let (mean, p25) = baseline
                .map(|b| (Some(b.mean_usd), Some(b.p25_usd)))
                .unwrap_or((None, None));

            // Best weekend price in last 90 days
            let prices = db
                .prices_for_route_by_days(&route.origin, dest, 90, None)
                .await
                .unwrap_or_default();

            let best_price = prices.iter().map(|p| p.price_usd).reduce(f64::min);
            let last_fetched = prices.first().map(|p| p.fetched_at.to_rfc3339());

            result.push(RouteResponse {
                route: format!("{}-{}", route.origin, dest),
                origin: route.origin.clone(),
                destination: dest.clone(),
                phase: phase.to_string(),
                days_of_data: days,
                last_fetched_at: last_fetched,
                best_weekend_price_usd: best_price,
                baseline_mean_usd: mean,
                baseline_p25_usd: p25,
            });
        }
    }

    Json(result)
}
