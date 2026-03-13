import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getRoutes } from '../api/client'
import type { RouteStatus } from '../api/types'

export const useRoutesStore = defineStore('routes', () => {
  const routes = ref<RouteStatus[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetch() {
    loading.value = true
    error.value = null
    try {
      routes.value = await getRoutes()
    } catch (e: any) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  return { routes, loading, error, fetch }
})
