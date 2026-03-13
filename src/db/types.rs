use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRow {
    pub id: i64,
    pub origin: String,
    pub destination: String,
    pub departure_date: NaiveDate,
    pub return_date: NaiveDate,
    pub price_usd: f64,
    pub source: String,
    pub is_scraped: bool,
    pub booking_url: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub id: Option<i64>,
    pub origin: String,
    pub destination: String,
    pub mean_usd: f64,
    pub median_usd: f64,
    pub p10_usd: f64,
    pub p25_usd: f64,
    pub stddev_usd: f64,
    pub sample_count: i64,
    pub days_of_data: i64,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Option<i64>,
    pub origin: String,
    pub destination: String,
    pub departure_date: NaiveDate,
    pub price_usd: f64,
    pub baseline_mean: f64,
    pub pct_below: f64,
    pub source: String,
    pub severity: String,
    pub booking_url: Option<String>,
    pub fingerprint: String,
    pub alerted_at: DateTime<Utc>,
}
