use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub origin: String,
    pub destinations: Vec<String>,  // ["NRT", "HND"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub routes: Vec<Route>,
    pub schedule_cron: String,
    pub window_days: u32,
    pub db_path: String,
    pub discord_webhook_url: Option<String>,
    pub log_level: String,
    pub api_port: u16,
    pub health_port: u16,
    pub detection: DetectionConfig,
    pub dedup: DedupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub stddev_multiplier: f64,
    pub p25_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupConfig {
    pub bucket_size: f64,
    pub realer_threshold: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            routes: default_routes(),
            schedule_cron: "0 0,12 * * *".to_string(),
            window_days: 90,
            db_path: "./data/flights.db".to_string(),
            discord_webhook_url: None,
            log_level: "info".to_string(),
            api_port: 3000,
            health_port: 8081,
            detection: DetectionConfig {
                stddev_multiplier: 1.5,
                p25_factor: 0.85,
            },
            dedup: DedupConfig {
                bucket_size: 50.0,
                realer_threshold: 0.10,
            },
        }
    }
}

pub fn default_routes() -> Vec<Route> {
    vec![
        Route { origin: "LAX".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "SFO".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "JFK".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "ORD".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "SEA".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "BOS".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "DFW".into(), destinations: vec!["NRT".into(), "HND".into()] },
        Route { origin: "MIA".into(), destinations: vec!["NRT".into(), "HND".into()] },
    ]
}

impl Config {
    /// Load from config.toml if present, then override with env vars.
    /// Falls back to defaults if file is missing.
    pub fn load() -> anyhow::Result<Self> {
        // Try to read config.toml
        let base: Config = if std::path::Path::new("config.toml").exists() {
            let content = std::fs::read_to_string("config.toml")?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Env overrides
        let mut cfg = base;
        if let Ok(val) = std::env::var("DB_PATH") { cfg.db_path = val; }
        if let Ok(val) = std::env::var("DISCORD_WEBHOOK_URL") { cfg.discord_webhook_url = Some(val); }
        if let Ok(val) = std::env::var("LOG_LEVEL") { cfg.log_level = val; }
        if let Ok(val) = std::env::var("API_PORT") { cfg.api_port = val.parse().unwrap_or(3000); }
        if let Ok(val) = std::env::var("HEALTH_PORT") { cfg.health_port = val.parse().unwrap_or(8081); }

        Ok(cfg)
    }
}
