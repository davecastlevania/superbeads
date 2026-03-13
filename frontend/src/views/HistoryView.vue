<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Price History</h1>

    <div class="flex flex-wrap gap-3 mb-6">
      <select v-model="selectedRoute" class="border border-gray-300 rounded-md px-3 py-1.5 text-sm">
        <option v-for="r in routes" :key="r" :value="r">{{ r }}</option>
      </select>
      <div class="flex rounded-md overflow-hidden border border-gray-300">
        <button
          v-for="d in dayOptions"
          :key="d.value"
          @click="selectedDays = d.value"
          class="px-3 py-1.5 text-sm"
          :class="selectedDays === d.value ? 'bg-blue-600 text-white' : 'bg-white text-gray-700 hover:bg-gray-50'"
        >{{ d.label }}</button>
      </div>
      <select v-model="selectedSource" class="border border-gray-300 rounded-md px-3 py-1.5 text-sm">
        <option value="all">All Sources</option>
        <option value="google_flights">Google Flights</option>
        <option value="skyscanner">Skyscanner</option>
        <option value="kayak">Kayak</option>
      </select>
    </div>

    <div v-if="loading" class="text-gray-500 text-sm py-8 text-center">Loading…</div>
    <div v-else-if="!prices.length" class="text-gray-400 text-sm py-8 text-center">
      No price data yet for this range.
    </div>
    <div v-else>
      <div class="bg-white rounded-xl border border-gray-200 p-4 mb-4">
        <canvas ref="chartCanvas" height="80" />
      </div>
      <div class="flex gap-4 text-sm text-gray-600">
        <span>Min: <strong>${{ stats.min }}</strong></span>
        <span>Max: <strong>${{ stats.max }}</strong></span>
        <span>Avg: <strong>${{ stats.avg }}</strong></span>
        <span>Samples: <strong>{{ stats.count }}</strong></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import 'chartjs-adapter-date-fns'
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { Chart, registerables } from 'chart.js'
import { getHistory } from '../api/client'
import type { PricePoint } from '../api/types'

Chart.register(...registerables)

const ROUTES = [
  'LAX-NRT','LAX-HND','SFO-NRT','SFO-HND','JFK-NRT','JFK-HND',
  'ORD-NRT','ORD-HND','SEA-NRT','SEA-HND','BOS-NRT','BOS-HND',
  'DFW-NRT','DFW-HND','MIA-NRT','MIA-HND',
]
const dayOptions = [
  { label: '7d', value: 7 },
  { label: '30d', value: 30 },
  { label: '90d', value: 90 },
  { label: 'All', value: 9999 },
]
const sourceColors: Record<string, string> = {
  google_flights: 'rgb(59,130,246)',
  skyscanner: 'rgb(249,115,22)',
  kayak: 'rgb(239,68,68)',
}

const routes = ref(ROUTES)
const selectedRoute = ref(ROUTES[0])
const selectedDays = ref(30)
const selectedSource = ref('all')
const prices = ref<PricePoint[]>([])
const loading = ref(false)
const chartCanvas = ref<HTMLCanvasElement | null>(null)
let chart: Chart | null = null

const stats = computed(() => {
  if (!prices.value.length) return { min: 0, max: 0, avg: 0, count: 0 }
  const vals = prices.value.map(p => p.price_usd)
  return {
    min: Math.round(Math.min(...vals)),
    max: Math.round(Math.max(...vals)),
    avg: Math.round(vals.reduce((a, b) => a + b, 0) / vals.length),
    count: vals.length,
  }
})

async function load() {
  loading.value = true
  try {
    const resp = await getHistory(selectedRoute.value, selectedDays.value, selectedSource.value)
    prices.value = resp.prices
    await nextTick()
    renderChart()
  } catch {
    prices.value = []
  } finally {
    loading.value = false
  }
}

function renderChart() {
  if (!chartCanvas.value) return
  chart?.destroy()

  const sources = [...new Set(prices.value.map(p => p.source))]
  const datasets = sources.map(src => ({
    label: src.replace('_', ' '),
    data: prices.value
      .filter(p => p.source === src)
      .map(p => ({ x: p.fetched_at, y: p.price_usd })),
    borderColor: sourceColors[src] ?? 'rgb(107,114,128)',
    backgroundColor: 'transparent',
    pointRadius: 3,
    tension: 0.2,
  }))

  chart = new Chart(chartCanvas.value, {
    type: 'line',
    data: { datasets },
    options: {
      responsive: true,
      scales: {
        x: { type: 'time', time: { unit: 'day' } },
        y: { beginAtZero: false, ticks: { callback: v => `$${v}` } },
      },
      plugins: { tooltip: { callbacks: {
        label: ctx => `$${Math.round(ctx.parsed.y as number)} — ${ctx.dataset.label}`
      }}},
    },
  })
}

watch([selectedRoute, selectedDays, selectedSource], load)
onMounted(load)
</script>
