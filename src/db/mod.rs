pub mod types;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;
use tracing::debug;

use crate::fetcher::types::PriceResult;
use types::{Alert, Baseline, PriceRow};

#[async_trait]
pub trait PriceRepository: Send + Sync {
    async fn insert_prices(&self, prices: &[PriceResult]) -> Result<()>;
    async fn prices_for_route(
        &self,
        origin: &str,
        dest: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<PriceRow>>;
    async fn upsert_baseline(&self, baseline: &Baseline) -> Result<()>;
    async fn baseline_for_route(&self, origin: &str, dest: &str) -> Result<Option<Baseline>>;
    async fn insert_alert(&self, alert: &Alert) -> Result<()>;
    async fn alert_exists(&self, fingerprint: &str) -> Result<bool>;
    async fn recent_alerts(&self, limit: u32) -> Result<Vec<Alert>>;
    async fn latest_alert_for_route_date(
        &self,
        origin: &str,
        dest: &str,
        departure_date: NaiveDate,
    ) -> Result<Option<Alert>>;
    async fn days_of_data_for_route(&self, origin: &str, dest: &str) -> Result<u32>;
    async fn prices_for_route_by_days(
        &self,
        origin: &str,
        dest: &str,
        days: u32,
        source_filter: Option<&str>,
    ) -> Result<Vec<PriceRow>>;
}

pub struct SqliteRepo {
    pool: SqlitePool,
}

impl SqliteRepo {
    pub async fn new(db_path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let url = format!("sqlite://{}?mode=rwc", db_path);
        let pool = SqlitePool::connect(&url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}

// Helper: parse datetime string from SQLite TEXT column
fn parse_dt(s: &str) -> DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive())
}

fn row_to_price_row(row: &sqlx::sqlite::SqliteRow) -> PriceRow {
    use sqlx::Row;
    PriceRow {
        id: row.get("id"),
        origin: row.get("origin"),
        destination: row.get("destination"),
        departure_date: parse_date(row.get("departure_date")),
        return_date: parse_date(row.get("return_date")),
        price_usd: row.get("price_usd"),
        source: row.get("source"),
        is_scraped: row.get::<i64, _>("is_scraped") != 0,
        booking_url: row.get("booking_url"),
        fetched_at: parse_dt(row.get("fetched_at")),
    }
}

fn row_to_alert(row: &sqlx::sqlite::SqliteRow) -> Alert {
    use sqlx::Row;
    Alert {
        id: Some(row.get("id")),
        origin: row.get("origin"),
        destination: row.get("destination"),
        departure_date: parse_date(row.get("departure_date")),
        price_usd: row.get("price_usd"),
        baseline_mean: row.get("baseline_mean"),
        pct_below: row.get("pct_below"),
        source: row.get("source"),
        severity: row.get("severity"),
        booking_url: row.get("booking_url"),
        fingerprint: row.get("fingerprint"),
        alerted_at: parse_dt(row.get("alerted_at")),
    }
}

#[async_trait]
impl PriceRepository for SqliteRepo {
    async fn insert_prices(&self, prices: &[PriceResult]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for p in prices {
            sqlx::query(
                "INSERT OR IGNORE INTO prices
                 (origin, destination, departure_date, return_date, price_usd,
                  source, is_scraped, booking_url, fetched_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&p.origin)
            .bind(&p.destination)
            .bind(p.departure_date.to_string())
            .bind(p.return_date.to_string())
            .bind(p.price_usd)
            .bind(p.source.to_string())
            .bind(p.is_scraped as i64)
            .bind(&p.booking_url)
            .bind(p.fetched_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        debug!(count = prices.len(), "Inserted prices");
        Ok(())
    }

    async fn prices_for_route(
        &self,
        origin: &str,
        dest: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<PriceRow>> {
        let rows = sqlx::query(
            "SELECT id, origin, destination, departure_date, return_date, price_usd,
                    source, is_scraped, booking_url, fetched_at
             FROM prices
             WHERE origin = ? AND destination = ? AND fetched_at >= ?
             ORDER BY fetched_at DESC",
        )
        .bind(origin)
        .bind(dest)
        .bind(since.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_price_row).collect())
    }

    async fn prices_for_route_by_days(
        &self,
        origin: &str,
        dest: &str,
        days: u32,
        source_filter: Option<&str>,
    ) -> Result<Vec<PriceRow>> {
        let since = Utc::now() - chrono::Duration::days(days as i64);
        let rows = if let Some(src) = source_filter {
            sqlx::query(
                "SELECT id, origin, destination, departure_date, return_date, price_usd,
                        source, is_scraped, booking_url, fetched_at
                 FROM prices
                 WHERE origin = ? AND destination = ? AND fetched_at >= ? AND source = ?
                 ORDER BY fetched_at DESC",
            )
            .bind(origin)
            .bind(dest)
            .bind(since.to_rfc3339())
            .bind(src)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, origin, destination, departure_date, return_date, price_usd,
                        source, is_scraped, booking_url, fetched_at
                 FROM prices
                 WHERE origin = ? AND destination = ? AND fetched_at >= ?
                 ORDER BY fetched_at DESC",
            )
            .bind(origin)
            .bind(dest)
            .bind(since.to_rfc3339())
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.iter().map(row_to_price_row).collect())
    }

    async fn upsert_baseline(&self, b: &Baseline) -> Result<()> {
        sqlx::query(
            "INSERT INTO baselines
             (origin, destination, mean_usd, median_usd, p10_usd, p25_usd,
              stddev_usd, sample_count, days_of_data, computed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(origin, destination) DO UPDATE SET
               mean_usd     = excluded.mean_usd,
               median_usd   = excluded.median_usd,
               p10_usd      = excluded.p10_usd,
               p25_usd      = excluded.p25_usd,
               stddev_usd   = excluded.stddev_usd,
               sample_count = excluded.sample_count,
               days_of_data = excluded.days_of_data,
               computed_at  = excluded.computed_at",
        )
        .bind(&b.origin)
        .bind(&b.destination)
        .bind(b.mean_usd)
        .bind(b.median_usd)
        .bind(b.p10_usd)
        .bind(b.p25_usd)
        .bind(b.stddev_usd)
        .bind(b.sample_count)
        .bind(b.days_of_data)
        .bind(b.computed_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn baseline_for_route(&self, origin: &str, dest: &str) -> Result<Option<Baseline>> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT id, origin, destination, mean_usd, median_usd, p10_usd, p25_usd,
                    stddev_usd, sample_count, days_of_data, computed_at
             FROM baselines WHERE origin = ? AND destination = ?",
        )
        .bind(origin)
        .bind(dest)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Baseline {
            id: Some(r.get("id")),
            origin: r.get("origin"),
            destination: r.get("destination"),
            mean_usd: r.get("mean_usd"),
            median_usd: r.get("median_usd"),
            p10_usd: r.get("p10_usd"),
            p25_usd: r.get("p25_usd"),
            stddev_usd: r.get("stddev_usd"),
            sample_count: r.get("sample_count"),
            days_of_data: r.get("days_of_data"),
            computed_at: parse_dt(r.get("computed_at")),
        }))
    }

    async fn insert_alert(&self, a: &Alert) -> Result<()> {
        sqlx::query(
            "INSERT INTO alerts
             (origin, destination, departure_date, price_usd, baseline_mean,
              pct_below, source, severity, booking_url, fingerprint, alerted_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&a.origin)
        .bind(&a.destination)
        .bind(a.departure_date.to_string())
        .bind(a.price_usd)
        .bind(a.baseline_mean)
        .bind(a.pct_below)
        .bind(&a.source)
        .bind(&a.severity)
        .bind(&a.booking_url)
        .bind(&a.fingerprint)
        .bind(a.alerted_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn alert_exists(&self, fingerprint: &str) -> Result<bool> {
        use sqlx::Row;
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM alerts WHERE fingerprint = ?")
            .bind(fingerprint)
            .fetch_one(&self.pool)
            .await?;
        let cnt: i64 = row.get("cnt");
        Ok(cnt > 0)
    }

    async fn recent_alerts(&self, limit: u32) -> Result<Vec<Alert>> {
        let rows = sqlx::query(
            "SELECT id, origin, destination, departure_date, price_usd, baseline_mean,
                    pct_below, source, severity, booking_url, fingerprint, alerted_at
             FROM alerts ORDER BY alerted_at DESC LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(row_to_alert).collect())
    }

    async fn latest_alert_for_route_date(
        &self,
        origin: &str,
        dest: &str,
        departure_date: NaiveDate,
    ) -> Result<Option<Alert>> {
        let row = sqlx::query(
            "SELECT id, origin, destination, departure_date, price_usd, baseline_mean,
                    pct_below, source, severity, booking_url, fingerprint, alerted_at
             FROM alerts
             WHERE origin = ? AND destination = ? AND departure_date = ?
             ORDER BY alerted_at DESC LIMIT 1",
        )
        .bind(origin)
        .bind(dest)
        .bind(departure_date.to_string())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(row_to_alert))
    }

    async fn days_of_data_for_route(&self, origin: &str, dest: &str) -> Result<u32> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT COUNT(DISTINCT DATE(fetched_at)) as cnt
             FROM prices WHERE origin = ? AND destination = ?",
        )
        .bind(origin)
        .bind(dest)
        .fetch_one(&self.pool)
        .await?;
        let cnt: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(cnt as u32)
    }
}
