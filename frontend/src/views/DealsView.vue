<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Deal Feed</h1>

    <div class="flex flex-wrap gap-3 mb-6">
      <select v-model="routeFilter" class="border border-gray-300 rounded-md px-3 py-1.5 text-sm">
        <option value="">All Routes</option>
        <option v-for="r in ROUTES" :key="r" :value="r">{{ r }}</option>
      </select>
      <select v-model="severityFilter" class="border border-gray-300 rounded-md px-3 py-1.5 text-sm">
        <option value="">All Severities</option>
        <option value="good_deal">Good Deal</option>
        <option value="exceptional_deal">Exceptional Deal</option>
      </select>
    </div>

    <div v-if="store.loading" class="text-gray-500 text-sm py-8 text-center">Loading…</div>

    <div v-else-if="!filtered.length" class="text-gray-400 text-sm py-8 text-center">
      <span v-if="store.deals.length === 0">
        No deals yet — the tracker is still learning your routes. Check back in 14 days.
      </span>
      <span v-else>No deals match your filters.</span>
    </div>

    <div v-else>
      <!-- Desktop table -->
      <div class="hidden md:block overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-200 text-left text-gray-500">
              <th class="pb-2 font-medium">Route</th>
              <th class="pb-2 font-medium">Departure</th>
              <th class="pb-2 font-medium cursor-pointer" @click="sortBy('price_usd')">Price ↕</th>
              <th class="pb-2 font-medium">Avg</th>
              <th class="pb-2 font-medium cursor-pointer" @click="sortBy('pct_below')">% Below ↕</th>
              <th class="pb-2 font-medium">Severity</th>
              <th class="pb-2 font-medium">Source</th>
              <th class="pb-2 font-medium">Alerted</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="deal in filtered" :key="deal.alerted_at + deal.route" class="border-b border-gray-100 hover:bg-gray-50">
              <td class="py-2 font-medium">{{ deal.origin }} → {{ deal.destination }}</td>
              <td class="py-2">{{ fmtDate(deal.departure_date) }}</td>
              <td class="py-2">
                <a v-if="deal.booking_url" :href="deal.booking_url" target="_blank" class="text-blue-600 hover:underline">
                  ${{ Math.round(deal.price_usd) }}
                </a>
                <span v-else>${{ Math.round(deal.price_usd) }}</span>
              </td>
              <td class="py-2 text-gray-500">${{ Math.round(deal.baseline_mean) }}</td>
              <td class="py-2 text-green-600 font-medium">{{ deal.pct_below.toFixed(1) }}%</td>
              <td class="py-2">
                <span
                  class="px-2 py-0.5 rounded-full text-xs font-medium"
                  :class="deal.severity === 'exceptional_deal'
                    ? 'bg-yellow-100 text-yellow-800'
                    : 'bg-green-100 text-green-700'"
                >
                  {{ deal.severity === 'exceptional_deal' ? 'Exceptional' : 'Good Deal' }}
                </span>
              </td>
              <td class="py-2 text-gray-500">{{ deal.source.replace('_', ' ') }}</td>
              <td class="py-2 text-gray-400">{{ formatAgo(deal.alerted_at) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Mobile cards -->
      <div class="md:hidden space-y-3">
        <div v-for="deal in filtered" :key="deal.alerted_at + deal.route" class="bg-white rounded-xl border border-gray-200 p-4">
          <div class="flex justify-between items-start mb-2">
            <span class="font-semibold">{{ deal.origin }} → {{ deal.destination }}</span>
            <span
              class="px-2 py-0.5 rounded-full text-xs font-medium"
              :class="deal.severity === 'exceptional_deal' ? 'bg-yellow-100 text-yellow-800' : 'bg-green-100 text-green-700'"
            >{{ deal.severity === 'exceptional_deal' ? 'Exceptional' : 'Good' }}</span>
          </div>
          <p class="text-sm text-gray-500">{{ fmtDate(deal.departure_date) }}</p>
          <p class="text-lg font-bold text-gray-900">
            <a v-if="deal.booking_url" :href="deal.booking_url" target="_blank" class="text-blue-600">${{ Math.round(deal.price_usd) }}</a>
            <span v-else>${{ Math.round(deal.price_usd) }}</span>
            <span class="text-sm font-normal text-green-600 ml-2">{{ deal.pct_below.toFixed(1) }}% off</span>
          </p>
        </div>
      </div>

      <div class="mt-4 text-sm text-gray-500">
        Showing {{ filtered.length }} of {{ store.total }} deals
      </div>
      <button
        v-if="store.total > limit"
        @click="loadMore"
        class="mt-3 px-4 py-2 text-sm border border-gray-300 rounded-md hover:bg-gray-50"
      >Load more</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useDealsStore } from '../stores/deals'

const ROUTES = [
  'LAX-NRT','LAX-HND','SFO-NRT','SFO-HND','JFK-NRT','JFK-HND',
  'ORD-NRT','ORD-HND','SEA-NRT','SEA-HND','BOS-NRT','BOS-HND',
  'DFW-NRT','DFW-HND','MIA-NRT','MIA-HND',
]

const store = useDealsStore()
const routeFilter = ref('')
const severityFilter = ref('')
const sortKey = ref<'price_usd' | 'pct_below'>('pct_below')
const sortDesc = ref(true)
const limit = ref(50)

const filtered = computed(() => {
  let result = [...store.deals]
  if (routeFilter.value) result = result.filter(d => d.route === routeFilter.value)
  if (severityFilter.value) result = result.filter(d => d.severity === severityFilter.value)
  result.sort((a, b) => {
    const diff = a[sortKey.value] - b[sortKey.value]
    return sortDesc.value ? -diff : diff
  })
  return result
})

function sortBy(key: 'price_usd' | 'pct_below') {
  if (sortKey.value === key) sortDesc.value = !sortDesc.value
  else { sortKey.value = key; sortDesc.value = true }
}

function fmtDate(s: string): string {
  return new Date(s).toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' })
}

function formatAgo(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime()
  const h = Math.floor(diff / 3_600_000)
  return h < 1 ? '<1h ago' : h < 24 ? `${h}h ago` : `${Math.floor(h / 24)}d ago`
}

async function loadMore() {
  limit.value += 50
  await store.fetch(limit.value, routeFilter.value || undefined)
}

watch([routeFilter, severityFilter], () => store.fetch(limit.value, routeFilter.value || undefined))
onMounted(() => store.fetch())
</script>
