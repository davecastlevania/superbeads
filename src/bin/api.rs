use std::sync::Arc;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use flight_tracker::{api, config::Config, db::SqliteRepo};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::load()?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.log_level));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .init();

    let db = Arc::new(SqliteRepo::new(&cfg.db_path).await?);
    let app = api::router(db);
    let addr = format!("0.0.0.0:{}", cfg.api_port);

    info!(event = "api_start", addr = addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
