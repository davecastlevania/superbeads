use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::Db;
use crate::db::PriceRepository;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub days: Option<u32>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub route: String,
    pub prices: Vec<PricePoint>,
}

#[derive(Debug, Serialize)]
pub struct PricePoint {
    pub fetched_at: String,
    pub departure_date: String,
    pub return_date: String,
    pub price_usd: f64,
    pub source: String,
    pub is_scraped: bool,
    pub booking_url: Option<String>,
}

pub async fn route_history(
    Path(route): Path<String>,
    Query(params): Query<HistoryQuery>,
    State(db): State<Db>,
) -> Result<Json<HistoryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let parts: Vec<&str> = route.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Route must be in format ORIGIN-DEST", "code": "BAD_ROUTE" })),
        ));
    }
    let (origin, dest) = (parts[0], parts[1]);
    let days = params.days.unwrap_or(30);
    let source_filter = params.source.as_deref().filter(|s| *s != "all");

    let rows = db
        .prices_for_route_by_days(origin, dest, days, source_filter)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string(), "code": "DB_ERROR" })),
            )
        })?;

    let prices = rows
        .into_iter()
        .map(|r| PricePoint {
            fetched_at: r.fetched_at.to_rfc3339(),
            departure_date: r.departure_date.to_string(),
            return_date: r.return_date.to_string(),
            price_usd: r.price_usd,
            source: r.source,
            is_scraped: r.is_scraped,
            booking_url: r.booking_url,
        })
        .collect();

    Ok(Json(HistoryResponse {
        route: route.to_uppercase(),
        prices,
    }))
}
