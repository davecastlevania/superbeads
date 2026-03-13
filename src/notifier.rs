use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::json;
use tracing::{error, info};

use crate::db::{types::Alert, PriceRepository};
use crate::detector::{Deal, DealSeverity};

const COLOR_GOOD_DEAL: u32 = 3_066_993;        // #2ECC71 green
const COLOR_EXCEPTIONAL_DEAL: u32 = 15_844_367; // #F1C40F gold

pub struct Notifier {
    webhook_url: String,
    client: reqwest::Client,
}

impl Notifier {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub fn from_env() -> Result<Self> {
        let url = std::env::var("DISCORD_WEBHOOK_URL")
            .map_err(|_| anyhow!("DISCORD_WEBHOOK_URL env var is required"))?;
        Ok(Self::new(url))
    }

    /// Send a deal alert to Discord. On success, inserts the alert record into the DB.
    pub async fn send_alert(
        &self,
        deal: &Deal,
        fingerprint: &str,
        repo: &dyn PriceRepository,
        is_test: bool,
    ) -> Result<()> {
        let payload = self.build_embed(deal, is_test);

        // Retry up to 3 times on 5xx
        let mut last_err = None;
        for attempt in 1..=3u8 {
            let resp = self
                .client
                .post(&self.webhook_url)
                .json(&payload)
                .send()
                .await
                .context("HTTP request failed")?;

            let status = resp.status();
            if status.is_success() || status.as_u16() == 204 {
                info!(
                    event = "alert_sent",
                    origin = deal.origin,
                    destination = deal.destination,
                    departure_date = %deal.departure_date,
                    price_usd = deal.price_usd,
                    severity = %deal.severity,
                    is_test,
                );

                if !is_test {
                    let alert = Alert {
                        id: None,
                        origin: deal.origin.clone(),
                        destination: deal.destination.clone(),
                        departure_date: deal.departure_date,
                        price_usd: deal.price_usd,
                        baseline_mean: deal.baseline_mean,
                        pct_below: deal.pct_below_mean,
                        source: deal.source.to_string(),
                        severity: deal.severity.to_string(),
                        booking_url: deal.booking_url.clone(),
                        fingerprint: fingerprint.to_string(),
                        alerted_at: Utc::now(),
                    };
                    repo.insert_alert(&alert).await?;
                }
                return Ok(());
            } else if status.as_u16() >= 500 {
                error!(status = status.as_u16(), attempt, "Discord 5xx, retrying");
                last_err = Some(anyhow!("Discord returned HTTP {}", status.as_u16()));
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt as u32))).await;
            } else {
                // 4xx — do not retry, do not insert alert row
                error!(status = status.as_u16(), "Discord rejected webhook payload");
                return Err(anyhow!("Discord returned HTTP {}", status.as_u16()));
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow!("Exhausted retries sending Discord alert")))
    }

    fn build_embed(&self, deal: &Deal, is_test: bool) -> serde_json::Value {
        let color = match deal.severity {
            DealSeverity::GoodDeal => COLOR_GOOD_DEAL,
            DealSeverity::ExceptionalDeal => COLOR_EXCEPTIONAL_DEAL,
        };

        let title_prefix = if is_test { "[TEST] " } else { "" };
        let title = format!(
            "{}✈️ Flight Deal: {} → {}",
            title_prefix, deal.origin, deal.destination
        );

        let book_link = deal
            .booking_url
            .as_deref()
            .map(|url| format!("[Book now]({})", url))
            .unwrap_or_else(|| "Search on Google Flights".to_string());

        json!({
            "embeds": [{
                "title": title,
                "color": color,
                "description": book_link,
                "fields": [
                    { "name": "Departure", "value": deal.departure_date.format("%a %b %d, %Y").to_string(), "inline": true },
                    { "name": "Return",    "value": deal.return_date.format("%a %b %d, %Y").to_string(),    "inline": true },
                    { "name": "Price",     "value": format!("${:.0}", deal.price_usd),                      "inline": true },
                    { "name": "Avg Price", "value": format!("${:.0}", deal.baseline_mean),                  "inline": true },
                    { "name": "Below Avg", "value": format!("{:.1}% off", deal.pct_below_mean),             "inline": true },
                    { "name": "Source",    "value": deal.source.to_string(),                                "inline": true },
                ],
                "footer": { "text": "Flight Deal Tracker" },
                "timestamp": Utc::now().to_rfc3339(),
            }]
        })
    }
}
