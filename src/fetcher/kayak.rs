use chrono::{NaiveDate, Utc};
use tracing::{debug, warn};

use super::types::{FetchError, PriceResult, Source};

pub struct KayakFetcher {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl KayakFetcher {
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
                // TODO: Implement real Kayak API call
                // GET https://www.kayak.com/api/...
                warn!(
                    source = "kayak",
                    origin,
                    destination,
                    "Kayak API not implemented, would use key"
                );
                Err(FetchError::NoResults {
                    origin: origin.to_string(),
                    destination: destination.to_string(),
                    date: departure_date,
                })
            }
            None => {
                debug!(
                    source = "kayak",
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
        // Kayak stubs are ~3% higher than Google's; add a 2-second delay to
        // simulate Kayak's bot-detection rate limiting behaviour.
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let price = self.stub_price(origin, destination);
        debug!(
            source = "kayak",
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
            source: Source::Kayak,
            fetched_at: Utc::now(),
            is_scraped: true,
            booking_url: Some(format!(
                "https://www.kayak.com/flights/{origin}-{destination}/{}/",
                departure_date.format("%Y-%m-%d")
            )),
        })
    }

    fn stub_price(&self, origin: &str, _destination: &str) -> f64 {
        // Stub prices ~3% higher than Google's
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
        (base * 1.03 * 100.0_f64).round() / 100.0
    }
}
