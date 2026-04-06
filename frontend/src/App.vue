<script setup>
import FileUpload from './components/FileUpload.vue'
import JobList from './components/JobList.vue'
import { useJobs } from './composables/useJobs.js'

const { jobs, loading, error, refresh } = useJobs()
</script>

<template>
  <div class="app">
    <header class="topbar">
      <div class="brand">
        <h1>Collapse</h1>
        <span class="tagline">File Compression Service</span>
      </div>
    </header>

    <main class="container">
      <div class="grid">
        <FileUpload @uploaded="refresh" />
        <JobList :jobs="jobs" :loading="loading" @refresh="refresh" />
      </div>
      <p v-if="error" class="global-error">Connection error: {{ error }}</p>
    </main>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
}

.topbar {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--card-bg);
}

.brand h1 {
  margin: 0;
  font-size: 1.4rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.tagline {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.container {
  max-width: 720px;
  margin: 0 auto;
  padding: 24px 16px;
}

.grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.global-error {
  text-align: center;
  color: var(--danger);
  font-size: 0.85rem;
  margin-top: 16px;
}
</style>
