import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getDeals } from '../api/client'
import type { Deal } from '../api/types'

export const useDealsStore = defineStore('deals', () => {
  const deals = ref<Deal[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const total = ref(0)

  async function fetch(limit = 50, route?: string) {
    loading.value = true
    error.value = null
    try {
      const result = await getDeals(limit, route)
      deals.value = result
      total.value = result.length
    } catch (e: any) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  return { deals, loading, error, total, fetch }
})
