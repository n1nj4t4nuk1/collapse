import { ref, onMounted, onUnmounted } from 'vue'
import { listJobs } from '../services/api.js'

export function useJobs(pollInterval = 2000) {
  const jobs = ref([])
  const loading = ref(false)
  const error = ref(null)
  let timer = null

  async function refresh() {
    try {
      error.value = null
      jobs.value = await listJobs()
    } catch (e) {
      error.value = e.message
    }
  }

  async function initialLoad() {
    loading.value = true
    await refresh()
    loading.value = false
  }

  function startPolling() {
    stopPolling()
    timer = setInterval(refresh, pollInterval)
  }

  function stopPolling() {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  onMounted(() => {
    initialLoad()
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return { jobs, loading, error, refresh }
}
