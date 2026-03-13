use chrono::{NaiveDate, Utc};
use tracing::{debug, warn};

use super::types::{FetchError, PriceResult, Source};

pub struct SkyscannerFetcher {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl SkyscannerFetcher {
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
                // TODO: Implement real Skyscanner API call
                // GET https://partners.api.skyscanner.net/apiservices/browseroutes/v1.0/...
                warn!(
                    source = "skyscanner",
                    origin,
                    destination,
                    "Skyscanner API not implemented, would use key"
                );
                Err(FetchError::NoResults {
                    origin: origin.to_string(),
                    destination: destination.to_string(),
                    date: departure_date,
                })
            }
            None => {
                debug!(
                    source = "skyscanner",
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
        // Skyscanner stubs are ~5% lower than Google's to reflect typical pricing
        let price = self.stub_price(origin, destination);
        debug!(
            source = "skyscanner",
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
            source: Source::Skyscanner,
            fetched_at: Utc::now(),
            is_scraped: true,
            booking_url: Some(format!(
                "https://www.skyscanner.com/transport/flights/{origin}/{destination}/{}/",
                departure_date.format("%y%m%d")
            )),
        })
    }

    fn stub_price(&self, origin: &str, _destination: &str) -> f64 {
        // Stub prices ~5% lower than Google's
        let base = match origin {
            "LAX" => 850.0,
            "SFO" => 820.0,
            "JFK" => 950.0,
            "ORD" => 920.0,
            "SEA" => 800.0,
            "BOS" => 980.0,
            "DFW" => 890.0,
            "MIA" => 1020.0,
            _ => 900.0,
        };
        (base * 0.95 * 100.0_f64).round() / 100.0
    }
}
