# Flight Deal Tracker

Monitors 8 US → Tokyo routes every 12 hours for weekend flights. Pulls prices from Google Flights, Skyscanner, and Kayak (with fallback scraping), stores full price history, learns your baseline over 14 days, and alerts you on Discord when a deal drops below the historical average — once per deal, no spam.

## Architecture

```
┌─────────────────────────────────────────┐
│  tracker (binary)                       │
│  ├── scheduler  — 12h cron              │
│  ├── fetcher    — 3 sources + scraping  │
│  ├── baseline   — rolling stats         │
│  ├── detector   — deal thresholds       │
│  ├── dedup      — fingerprint cache     │
│  └── notifier   — Discord webhooks      │
└────────────────┬────────────────────────┘
                 │ SQLite (./data/flights.db)
┌────────────────┴────────────────────────┐
│  api (binary)  — Axum REST on :3000     │
└────────────────┬────────────────────────┘
                 │ /api/*
┌────────────────┴────────────────────────┐
│  frontend/     — Vue 3 + Vite on Deno   │
│  ├── Routes overview dashboard          │
│  ├── Price history charts               │
│  └── Deal feed                          │
└─────────────────────────────────────────┘
```

## Prerequisites

- [Rust](https://rustup.rs/) 1.82+
- [Deno](https://deno.com/) 2.x (frontend only)

## Running

Start all three processes in separate terminals:

```bash
# 1. REST API (port 3000)
cargo run --bin api

# 2. Scheduler + tracker
cargo run --bin tracker run

# 3. Frontend dev server (port 5173)
cd frontend && deno task dev
```

Open [http://localhost:5173](http://localhost:5173).

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `DISCORD_WEBHOOK_URL` | For alerts | — | Discord incoming webhook URL |
| `DB_PATH` | No | `./data/flights.db` | SQLite database path |
| `API_PORT` | No | `3000` | REST API listen port |
| `HEALTH_PORT` | No | `8081` | Health watchdog port |
| `LOG_LEVEL` | No | `info` | `trace` / `debug` / `info` / `warn` / `error` |
| `GOOGLE_FLIGHTS_API_KEY` | No | — | Enables Google Flights API (falls back to scraping if absent) |
| `SKYSCANNER_API_KEY` | No | — | Enables Skyscanner API |
| `KAYAK_API_KEY` | No | — | Enables Kayak API |

## CLI Commands

```bash
# Show status for all routes (phase, data age, best price)
cargo run --bin tracker status

# Price history for a route
cargo run --bin tracker history LAX-NRT
cargo run --bin tracker history LAX-NRT --days 60

# Manually trigger a fetch cycle (no alerts sent — useful for seeding)
cargo run --bin tracker backfill
cargo run --bin tracker backfill --route LAX-NRT

# Send a test Discord alert without a real check cycle
cargo run --bin tracker test-alert --route LAX-NRT --price 599 --baseline 890
```

## Seeding the Learning Window

The tracker requires **14 days of price data** before it starts sending alerts. To seed it faster, run backfill multiple times:

```bash
for i in $(seq 1 14); do cargo run --bin tracker backfill; sleep 2; done
```

## Routes Monitored

All routes query both NRT (Narita) and HND (Haneda) and take the lower price.

| Origin | City |
|--------|------|
| LAX | Los Angeles |
| SFO | San Francisco |
| JFK | New York |
| ORD | Chicago |
| SEA | Seattle |
| BOS | Boston |
| DFW | Dallas |
| MIA | Miami |

## Deal Detection

A price is flagged as a deal when either condition is met:

- **Good deal** — price < mean − 1.5 × stddev
- **Exceptional deal** — price < p25 × 0.85 (and both conditions met)

Thresholds are configurable in `config.toml`:

```toml
[detection]
stddev_multiplier = 1.5
p25_factor = 0.85

[dedup]
bucket_size = 50        # $50 price buckets for fingerprinting
realer_threshold = 0.10 # re-alert only if price drops another 10%
```

## REST API

The API server exposes four endpoints consumed by the frontend:

```
GET /api/routes                          — all routes with phase + current prices
GET /api/routes/:route/history?days=&source=  — price time-series
GET /api/deals?limit=&route=             — recent alerts
GET /api/health                          — scheduler heartbeat
```

## Frontend Build

```bash
cd frontend
deno task build   # outputs to frontend/dist/
deno task preview # preview the production build
```

Set `VITE_API_BASE_URL` to point the frontend at a non-local API server:

```bash
VITE_API_BASE_URL=https://api.example.com deno task build
```

## Project Structure

```
├── src/
│   ├── bin/
│   │   ├── tracker.rs   — CLI entry point
│   │   └── api.rs       — API server entry point
│   ├── api/             — Axum route handlers
│   ├── cli/             — status / history / backfill commands
│   ├── fetcher/         — Google Flights, Skyscanner, Kayak adapters
│   ├── db/              — SQLite repository + types
│   ├── config.rs        — Config struct + loader
│   ├── dates.rs         — Weekend date generator
│   ├── scheduler.rs     — 12h cron scheduler
│   ├── baseline.rs      — Statistical baseline computation
│   ├── detector.rs      — Deal detection engine
│   ├── dedup.rs         — Alert deduplication
│   ├── notifier.rs      — Discord webhook sender
│   └── health.rs        — Watchdog health status
├── migrations/
│   └── 001_initial.sql  — prices, baselines, alerts tables
├── frontend/            — Vue 3 + Vite + Tailwind dashboard
└── data/                — SQLite DB and last_run file (gitignored)
```
