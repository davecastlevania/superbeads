pub mod deals;
pub mod health;
pub mod history;
pub mod routes;

use std::sync::Arc;

use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::db::SqliteRepo;

pub type Db = Arc<SqliteRepo>;

pub fn router(db: Db) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/routes", get(routes::list_routes))
        .route("/api/routes/:route/history", get(history::route_history))
        .route("/api/deals", get(deals::list_deals))
        .route("/api/health", get(health::health_check))
        .with_state(db)
        .layer(cors)
}
