const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8000'

async function request(path, options = {}) {
  const res = await fetch(`${API_BASE}${path}`, options)
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    const error = new Error(body.detail || `HTTP ${res.status}`)
    error.status = res.status
    throw error
  }
  return res
}

export async function uploadFile(file, algorithm = '7z', level = 5) {
  const form = new FormData()
  form.append('file', file)
  form.append('algorithm', algorithm)
  form.append('level', level.toString())
  const res = await request('/files', { method: 'POST', body: form })
  return res.json()
}

export async function listJobs() {
  const res = await request('/files')
  return res.json()
}

export async function getJobStatus(jobId) {
  const res = await request(`/files/${jobId}/status`)
  return res.json()
}

export async function downloadFile(jobId) {
  const res = await fetch(`${API_BASE}/files/${jobId}/download`)
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error(body.detail || `HTTP ${res.status}`)
  }
  const blob = await res.blob()
  const disposition = res.headers.get('content-disposition') || ''
  const match = disposition.match(/filename="?(.+?)"?$/)
  const filename = match ? match[1] : `archive-${jobId}`
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

export async function deleteJob(jobId) {
  const res = await request(`/files/${jobId}`, { method: 'DELETE' })
  return res.json()
}

export async function deleteCompleted() {
  const res = await request('/files/completed', { method: 'DELETE' })
  return res.json()
}
