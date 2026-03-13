use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Source {
    GoogleFlights,
    Skyscanner,
    Kayak,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::GoogleFlights => write!(f, "google_flights"),
            Source::Skyscanner => write!(f, "skyscanner"),
            Source::Kayak => write!(f, "kayak"),
        }
    }
}

impl Source {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "google_flights" => Some(Self::GoogleFlights),
            "skyscanner" => Some(Self::Skyscanner),
            "kayak" => Some(Self::Kayak),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResult {
    pub origin: String,
    pub destination: String,
    pub departure_date: NaiveDate,
    pub return_date: NaiveDate,
    pub price_usd: f64,
    pub source: Source,
    pub fetched_at: DateTime<Utc>,
    pub is_scraped: bool,
    pub booking_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Rate limited by {src}")]
    RateLimited { src: String },
    #[error("HTTP {status} from {src}: {message}")]
    HttpError {
        src: String,
        status: u16,
        message: String,
    },
    #[error("Auth failed for {src}")]
    AuthError { src: String },
    #[error("Scraping failed for {src}: {message}")]
    ScrapingError { src: String, message: String },
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("No results for {origin}-{destination} on {date}")]
    NoResults {
        origin: String,
        destination: String,
        date: NaiveDate,
    },
}
