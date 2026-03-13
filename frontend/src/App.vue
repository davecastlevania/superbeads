<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-14 items-center">
          <div class="flex items-center gap-6">
            <span class="font-semibold text-gray-900 text-sm">✈️ Flight Deal Tracker</span>
            <router-link
              v-for="link in nav"
              :key="link.to"
              :to="link.to"
              class="text-sm text-gray-500 hover:text-gray-900 transition-colors"
              active-class="text-blue-600 font-medium"
            >{{ link.label }}</router-link>
          </div>
          <span
            class="inline-flex items-center gap-1.5 text-xs"
            :class="health.status === 'ok' ? 'text-green-600' : 'text-red-600'"
          >
            <span
              class="w-2 h-2 rounded-full"
              :class="health.status === 'ok' ? 'bg-green-500' : 'bg-red-500'"
            />
            {{ health.status === 'ok' ? 'Live' : health.status }}
          </span>
        </div>
      </div>
    </nav>
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import { getHealth } from './api/client'

const nav = [
  { to: '/', label: 'Routes' },
  { to: '/history', label: 'Price History' },
  { to: '/deals', label: 'Deal Feed' },
]

const health = reactive({ status: 'unknown' })

onMounted(async () => {
  try {
    const h = await getHealth()
    health.status = h.status
  } catch {
    health.status = 'error'
  }
})
</script>
