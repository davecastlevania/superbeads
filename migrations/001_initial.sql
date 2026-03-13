-- Flight Deal Tracker: initial schema

CREATE TABLE IF NOT EXISTS prices (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    origin         TEXT    NOT NULL,
    destination    TEXT    NOT NULL,
    departure_date TEXT    NOT NULL,  -- ISO date: YYYY-MM-DD
    return_date    TEXT    NOT NULL,  -- ISO date: YYYY-MM-DD
    price_usd      REAL    NOT NULL,
    source         TEXT    NOT NULL,  -- 'google_flights' | 'skyscanner' | 'kayak'
    is_scraped     INTEGER NOT NULL DEFAULT 0,
    booking_url    TEXT,
    fetched_at     TEXT    NOT NULL   -- RFC3339 datetime
);

CREATE INDEX IF NOT EXISTS idx_prices_route_fetched
    ON prices (origin, destination, fetched_at);

CREATE INDEX IF NOT EXISTS idx_prices_departure
    ON prices (origin, destination, departure_date);

-- ------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS baselines (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    origin       TEXT NOT NULL,
    destination  TEXT NOT NULL,
    mean_usd     REAL NOT NULL,
    median_usd   REAL NOT NULL,
    p10_usd      REAL NOT NULL,
    p25_usd      REAL NOT NULL,
    stddev_usd   REAL NOT NULL,
    sample_count INTEGER NOT NULL,
    days_of_data INTEGER NOT NULL,
    computed_at  TEXT NOT NULL,       -- RFC3339 datetime
    UNIQUE (origin, destination)      -- upserted on each recompute
);

-- ------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS alerts (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    origin         TEXT NOT NULL,
    destination    TEXT NOT NULL,
    departure_date TEXT NOT NULL,     -- ISO date
    price_usd      REAL NOT NULL,
    baseline_mean  REAL NOT NULL,
    pct_below      REAL NOT NULL,
    source         TEXT NOT NULL,
    severity       TEXT NOT NULL,     -- 'good_deal' | 'exceptional_deal'
    booking_url    TEXT,
    fingerprint    TEXT NOT NULL UNIQUE,
    alerted_at     TEXT NOT NULL      -- RFC3339 datetime
);

CREATE INDEX IF NOT EXISTS idx_alerts_fingerprint
    ON alerts (fingerprint);

CREATE INDEX IF NOT EXISTS idx_alerts_route_date
    ON alerts (origin, destination, departure_date);
