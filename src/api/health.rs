use axum::{http::StatusCode, Json};

pub async fn health_check() -> (StatusCode, Json<crate::health::HealthStatus>) {
    let status = crate::health::get_health().await;
    let http_status = if status.status == "ok" || status.status == "no_runs_yet" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (http_status, Json(status))
}
