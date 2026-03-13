use anyhow::Result;
use chrono::Utc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

pub struct Scheduler {
    sched: JobScheduler,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let sched = JobScheduler::new().await?;
        Ok(Self { sched })
    }

    /// Add a job that runs at 00:00 and 12:00 UTC every day.
    /// The callback receives no args; it should trigger the fetch pipeline.
    pub async fn add_check_job<F, Fut>(&mut self, f: F) -> Result<()>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        // Cron: sec min hour day month weekday
        // Run at 00:00 and 12:00 UTC
        let job = Job::new_async("0 0 0,12 * * *", move |_uuid, _lock| {
            let f = f.clone();
            Box::pin(async move {
                info!(event = "run_start", timestamp = %Utc::now());
                f().await;
            })
        })?;
        self.sched.add(job).await?;
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        self.sched.start().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.sched.shutdown().await?;
        Ok(())
    }
}

/// Writes the current timestamp to ./data/last_run for the watchdog.
pub async fn record_last_run() -> Result<()> {
    tokio::fs::create_dir_all("./data").await?;
    let now = Utc::now().to_rfc3339();
    tokio::fs::write("./data/last_run", now).await?;
    Ok(())
}

/// Returns the last run time from ./data/last_run, if it exists.
pub async fn read_last_run() -> Option<chrono::DateTime<Utc>> {
    let content = tokio::fs::read_to_string("./data/last_run").await.ok()?;
    chrono::DateTime::parse_from_rfc3339(content.trim()).ok().map(|dt| dt.with_timezone(&Utc))
}
