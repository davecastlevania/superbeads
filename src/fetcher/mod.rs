pub mod google;
pub mod kayak;
pub mod skyscanner;
pub mod types;

use chrono::NaiveDate;
use tracing::{info, warn};

use types::{FetchError, PriceResult};

pub struct Fetcher {
    google: google::GoogleFlightsFetcher,
    skyscanner: skyscanner::SkyscannerFetcher,
    kayak: kayak::KayakFetcher,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            google: google::GoogleFlightsFetcher::new(
                std::env::var("GOOGLE_FLIGHTS_API_KEY").ok(),
            ),
            skyscanner: skyscanner::SkyscannerFetcher::new(
                std::env::var("SKYSCANNER_API_KEY").ok(),
            ),
            kayak: kayak::KayakFetcher::new(std::env::var("KAYAK_API_KEY").ok()),
        }
    }

    /// Fetch prices from all sources for a single (origin, destination, dates) pair.
    /// Returns all successful results (may be 0-3 per date).
    pub async fn fetch_all(
        &self,
        origin: &str,
        destination: &str,
        departure_date: NaiveDate,
        return_date: NaiveDate,
    ) -> Vec<PriceResult> {
        let mut results = Vec::new();

        // Try each source independently; failures are logged, not propagated
        match self
            .google
            .fetch(origin, destination, departure_date, return_date)
            .await
        {
            Ok(r) => results.push(r),
            Err(FetchError::NoResults { .. }) => {}
            Err(e) => warn!(source = "google_flights", error = %e, "Fetch failed"),
        }

        match self
            .skyscanner
            .fetch(origin, destination, departure_date, return_date)
            .await
        {
            Ok(r) => results.push(r),
            Err(FetchError::NoResults { .. }) => {}
            Err(e) => warn!(source = "skyscanner", error = %e, "Fetch failed"),
        }

        match self
            .kayak
            .fetch(origin, destination, departure_date, return_date)
            .await
        {
            Ok(r) => results.push(r),
            Err(FetchError::NoResults { .. }) => {}
            Err(e) => warn!(source = "kayak", error = %e, "Fetch failed"),
        }

        info!(
            origin,
            destination,
            departure_date = %departure_date,
            results_count = results.len(),
            "Fetch complete for route/date"
        );

        results
    }

    /// Fetch all prices for a route across multiple departure dates.
    pub async fn fetch_route(
        &self,
        origin: &str,
        destinations: &[String],
        departure_dates: &[NaiveDate],
    ) -> Vec<PriceResult> {
        let mut all_results = Vec::new();

        for destination in destinations {
            for &departure_date in departure_dates {
                let return_date = crate::dates::return_date(departure_date);
                let results = self
                    .fetch_all(origin, destination, departure_date, return_date)
                    .await;
                all_results.extend(results);
            }
        }

        all_results
    }
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}
