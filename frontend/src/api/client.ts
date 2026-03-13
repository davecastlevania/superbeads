import type { RouteStatus, HistoryResponse, Deal, HealthResponse } from './types'

const BASE = (import.meta as any).env?.VITE_API_BASE_URL ?? ''

async function get<T>(path: string): Promise<T> {
  const resp = await fetch(`${BASE}${path}`)
  if (!resp.ok) throw new Error(`HTTP ${resp.status}: ${path}`)
  return resp.json()
}

export async function getRoutes(): Promise<RouteStatus[]> {
  return get('/api/routes')
}

export async function getHistory(
  route: string,
  days: number,
  source: string
): Promise<HistoryResponse> {
  const params = new URLSearchParams({ days: String(days), source })
  return get(`/api/routes/${route}/history?${params}`)
}

export async function getDeals(limit: number, route?: string): Promise<Deal[]> {
  const params = new URLSearchParams({ limit: String(limit) })
  if (route) params.set('route', route)
  return get(`/api/deals?${params}`)
}

export async function getHealth(): Promise<HealthResponse> {
  return get('/api/health')
}
