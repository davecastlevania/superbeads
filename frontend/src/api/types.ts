export interface RouteStatus {
  route: string
  origin: string
  destination: string
  phase: 'active' | 'learning'
  days_of_data: number
  last_fetched_at: string | null
  best_weekend_price_usd: number | null
  baseline_mean_usd: number | null
  baseline_p25_usd: number | null
}

export interface PricePoint {
  fetched_at: string
  departure_date: string
  return_date: string
  price_usd: number
  source: string
  is_scraped: boolean
  booking_url: string | null
}

export interface HistoryResponse {
  route: string
  prices: PricePoint[]
}

export interface Deal {
  route: string
  origin: string
  destination: string
  departure_date: string
  price_usd: number
  baseline_mean: number
  pct_below: number
  severity: 'good_deal' | 'exceptional_deal'
  source: string
  booking_url: string | null
  alerted_at: string
}

export interface HealthResponse {
  status: string
  last_run: string | null
  next_run: string | null
  hours_since_last_run: number | null
}
