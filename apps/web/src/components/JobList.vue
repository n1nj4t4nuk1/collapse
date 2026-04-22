<script setup>
import { downloadFile, deleteJob, deleteCompleted } from '../services/api.js'

const props = defineProps({
  jobs: { type: Array, required: true },
  loading: { type: Boolean, default: false },
})

const emit = defineEmits(['refresh'])

const statusConfig = {
  queued: { label: 'Queued', class: 'status-queued' },
  compressing: { label: 'Compressing', class: 'status-compressing' },
  completed: { label: 'Completed', class: 'status-completed' },
  failed: { label: 'Failed', class: 'status-failed' },
}

function statusInfo(status) {
  return statusConfig[status] || { label: status, class: '' }
}

function formatDate(iso) {
  return new Date(iso).toLocaleString()
}

async function download(jobId) {
  try {
    await downloadFile(jobId)
  } catch (e) {
    alert(e.message)
  }
}

async function remove(jobId) {
  try {
    await deleteJob(jobId)
    emit('refresh')
  } catch (e) {
    alert(e.message)
  }
}

async function removeCompleted() {
  try {
    const result = await deleteCompleted()
    emit('refresh')
    if (result.deleted_jobs === 0) {
      alert('No completed jobs to delete.')
    }
  } catch (e) {
    alert(e.message)
  }
}

const completedCount = () => props.jobs.filter(j => j.status === 'completed').length
</script>

<template>
  <section class="jobs-card">
    <div class="header">
      <h2>Jobs</h2>
      <button
        v-if="completedCount() > 0"
        class="btn-text danger"
        @click="removeCompleted"
      >
        Clear completed ({{ completedCount() }})
      </button>
    </div>

    <div v-if="loading" class="empty">Loading...</div>
    <div v-else-if="jobs.length === 0" class="empty">
      No compression jobs yet. Upload a file to get started.
    </div>

    <TransitionGroup v-else name="list" tag="ul" class="job-list">
      <li v-for="job in jobs" :key="job.job_id" class="job-item">
        <div class="job-main">
          <div class="job-name">
            <span class="filename">{{ job.filename }}</span>
            <span class="algo-badge">{{ job.algorithm.toUpperCase() }} L{{ job.level }}</span>
          </div>
          <div class="job-meta">
            <span :class="['status', statusInfo(job.status).class]">
              {{ statusInfo(job.status).label }}
            </span>
            <span class="date">{{ formatDate(job.created_at) }}</span>
          </div>
        </div>
        <div class="job-actions">
          <button
            v-if="job.status === 'completed'"
            class="btn-icon"
            title="Download"
            @click="download(job.job_id)"
          >
            ⬇
          </button>
          <button
            v-if="job.status !== 'compressing' && job.status !== 'queued'"
            class="btn-icon danger"
            title="Delete"
            @click="remove(job.job_id)"
          >
            ✕
          </button>
        </div>
      </li>
    </TransitionGroup>
  </section>
</template>

<style scoped>
.jobs-card {
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.header h2 {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
}

.btn-text {
  background: none;
  border: none;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.15s;
}
.btn-text.danger {
  color: var(--danger);
}
.btn-text.danger:hover {
  background: var(--danger-bg);
}

.empty {
  text-align: center;
  color: var(--text-muted);
  padding: 32px 0;
  font-size: 0.9rem;
}

.job-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.job-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: var(--item-bg);
  border-radius: 8px;
  transition: all 0.2s;
}
.job-item:hover {
  background: var(--item-hover);
}

.job-main {
  flex: 1;
  min-width: 0;
}

.job-name {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}
.filename {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.algo-badge {
  font-size: 0.7rem;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--badge-bg);
  color: var(--text-muted);
  white-space: nowrap;
}

.job-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 0.8rem;
}

.status {
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.75rem;
}
.status-queued { background: #fef3cd; color: #856404; }
.status-compressing { background: #cce5ff; color: #004085; }
.status-completed { background: #d4edda; color: #155724; }
.status-failed { background: #f8d7da; color: #721c24; }

.date {
  color: var(--text-muted);
}

.job-actions {
  display: flex;
  gap: 6px;
  margin-left: 12px;
}

.btn-icon {
  background: none;
  border: 1px solid var(--border);
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.95rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.btn-icon:hover {
  background: var(--item-hover);
}
.btn-icon.danger:hover {
  border-color: var(--danger);
  color: var(--danger);
  background: var(--danger-bg);
}

/* Transition animations */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}
.list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}
.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
