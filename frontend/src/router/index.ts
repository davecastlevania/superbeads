import { createRouter, createWebHistory } from 'vue-router'
import RoutesView from '../views/RoutesView.vue'
import HistoryView from '../views/HistoryView.vue'
import DealsView from '../views/DealsView.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: RoutesView },
    { path: '/history', component: HistoryView },
    { path: '/deals', component: DealsView },
  ],
})
