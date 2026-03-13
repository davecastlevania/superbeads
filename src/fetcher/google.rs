use chrono::{NaiveDate, Utc};
use tracing::{debug, warn};

use super::types::{FetchError, PriceResult, Source};

pub struct GoogleFlightsFetcher {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl GoogleFlightsFetcher {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn fetch(
        &self,
        origin: &str,
        destination: &str,
        departure_date: NaiveDate,
        return_date: NaiveDate,
    ) -> Result<PriceResult, FetchError> {
        match &self.api_key {
            Some(_key) => {
                // TODO: Implement real Google Flights API call
                // POST https://www.googleapis.com/qpxExpress/v1/trips/search
                // For now return a structured error indicating not yet implemented
                warn!(
                    source = "google_flights",
                    origin,
                    destination,
                    "Google Flights API not implemented, would use key"
                );
                Err(FetchError::NoResults {
                    origin: origin.to_string(),
                    destination: destination.to_string(),
                    date: departure_date,
                })
            }
            None => {
                debug!(
                    source = "google_flights",
                    "No API key, attempting scrape fallback"
                );
                self.scrape(origin, destination, departure_date, return_date)
                    .await
            }
        }
    }

    async fn scrape(
        &self,
        origin: &str,
        destination: &str,
        departure_date: NaiveDate,
        return_date: NaiveDate,
    ) -> Result<PriceResult, FetchError> {
        // TODO: Implement real scraping via headless browser
        // For now: stub that returns a plausible price for testing
        let price = self.stub_price(origin, destination);
        debug!(
            source = "google_flights",
            origin,
            destination,
            price,
            "Scrape stub returning test price"
        );
        Ok(PriceResult {
            origin: origin.to_string(),
            destination: destination.to_string(),
            departure_date,
            return_date,
            price_usd: price,
            source: Source::GoogleFlights,
            fetched_at: Utc::now(),
            is_scraped: true,
            booking_url: Some(format!(
                "https://flights.google.com/search?q=flights+{origin}+to+{destination}+{departure_date}"
            )),
        })
    }

    fn stub_price(&self, origin: &str, _destination: &str) -> f64 {
        // Deterministic stub prices per origin for testing
        match origin {
            "LAX" => 850.0,
            "SFO" => 820.0,
            "JFK" => 950.0,
            "ORD" => 920.0,
            "SEA" => 800.0,
            "BOS" => 980.0,
            "DFW" => 890.0,
            "MIA" => 1020.0,
            _ => 900.0,
        }
    }
}
