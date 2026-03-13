<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Routes Overview</h1>

    <div v-if="store.loading" class="text-gray-500 text-sm">Loading routes…</div>
    <div v-else-if="store.error" class="text-red-600 text-sm">{{ store.error }}</div>

    <div v-else-if="allLearning" class="bg-blue-50 border border-blue-200 rounded-lg p-4 text-blue-800 text-sm">
      Learning in progress — check back in 14 days.
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div
        v-for="route in sorted"
        :key="route.route"
        class="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden"
        :class="route.phase === 'learning' ? 'opacity-60' : ''"
      >
        <div class="px-4 pt-4 pb-3 flex items-center justify-between">
          <span class="font-semibold text-gray-900">{{ route.origin }} → {{ route.destination }}</span>
          <span
            class="text-xs font-medium px-2 py-0.5 rounded-full"
            :class="route.phase === 'active' ? 'bg-green-100 text-green-700' : 'bg-yellow-100 text-yellow-700'"
          >
            {{ route.phase === 'active' ? 'ACTIVE' : `Day ${route.days_of_data} of 14` }}
          </span>
        </div>

        <div v-if="route.phase === 'learning'" class="px-4 pb-1">
          <div class="w-full bg-gray-100 rounded-full h-1.5">
            <div
              class="bg-yellow-400 h-1.5 rounded-full transition-all"
              :style="{ width: `${Math.min(route.days_of_data / 14 * 100, 100)}%` }"
            />
          </div>
          <p class="text-xs text-gray-400 mt-1">Collecting data…</p>
        </div>

        <div class="px-4 pb-4 grid grid-cols-2 gap-x-4 gap-y-1 text-sm mt-1">
          <div class="text-gray-500">Last checked</div>
          <div class="text-gray-700 text-right">{{ formatAgo(route.last_fetched_at) }}</div>

          <div class="text-gray-500">Best price (90d)</div>
          <div class="text-right font-medium" :class="isDeal(route) ? 'text-green-600' : 'text-gray-900'">
            {{ route.best_weekend_price_usd != null ? `$${Math.round(route.best_weekend_price_usd)}` : '—' }}
          </div>

          <template v-if="route.phase === 'active' && route.baseline_mean_usd">
            <div class="text-gray-500">Avg price</div>
            <div class="text-gray-700 text-right">${{ Math.round(route.baseline_mean_usd) }}</div>
          </template>

          <template v-if="isDeal(route)">
            <div class="text-gray-500">Deal</div>
            <div class="text-green-600 text-right font-medium">
              {{ dealPct(route) }}% below avg
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRoutesStore } from '../stores/routes'
import type { RouteStatus } from '../api/types'

const store = useRoutesStore()

const sorted = computed(() =>
  [...store.routes].sort((a, b) => {
    if (a.phase !== b.phase) return a.phase === 'active' ? -1 : 1
    return dealPct(b) - dealPct(a)
  })
)

const allLearning = computed(() =>
  store.routes.length > 0 && store.routes.every(r => r.phase === 'learning')
)

function isDeal(r: RouteStatus): boolean {
  if (r.phase !== 'active' || !r.best_weekend_price_usd || !r.baseline_mean_usd) return false
  return r.best_weekend_price_usd < r.baseline_mean_usd * 0.9
}

function dealPct(r: RouteStatus): number {
  if (!r.best_weekend_price_usd || !r.baseline_mean_usd) return 0
  return Math.round((1 - r.best_weekend_price_usd / r.baseline_mean_usd) * 100)
}

function formatAgo(ts: string | null): string {
  if (!ts) return 'never'
  const diff = Date.now() - new Date(ts).getTime()
  const h = Math.floor(diff / 3_600_000)
  if (h < 1) return '<1h ago'
  if (h < 24) return `${h}h ago`
  return `${Math.floor(h / 24)}d ago`
}

onMounted(() => {
  store.fetch()
  const interval = setInterval(() => store.fetch(), 5 * 60 * 1000)
  onUnmounted(() => clearInterval(interval))
})
</script>
