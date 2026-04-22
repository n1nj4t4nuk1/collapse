<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { getCurrentWebview } from '@tauri-apps/api/webview'

const ARCHIVE_EXTENSIONS = ['zip', '7z']

const filePath = ref(null)
const fileName = ref('')
const dragging = ref(false)
const processing = ref(false)
const result = ref(null)
const error = ref(null)
const protocol = ref('zip')
const level = ref(5)

const isArchive = computed(() => {
  if (!fileName.value) return false
  const ext = fileName.value.split('.').pop().toLowerCase()
  return ARCHIVE_EXTENSIONS.includes(ext)
})

const mode = computed(() => {
  if (!filePath.value) return 'idle'
  return isArchive.value ? 'extract' : 'compress'
})

function reset() {
  filePath.value = null
  fileName.value = ''
  result.value = null
  error.value = null
}

function selectFile(path) {
  result.value = null
  error.value = null
  filePath.value = path
  fileName.value = path.split('/').pop().split('\\').pop()
}

async function browse() {
  const selected = await open({ multiple: false })
  if (selected) selectFile(selected)
}

async function doCompress() {
  processing.value = true
  error.value = null
  result.value = null
  try {
    const ext = protocol.value === '7z' ? '7z' : 'zip'
    const defaultName = `${fileName.value}.${ext}`
    const savePath = await save({ defaultPath: defaultName })
    if (!savePath) {
      processing.value = false
      return
    }
    const output = await invoke('compress_file', {
      file: filePath.value,
      output: savePath,
      protocol: protocol.value,
      level: level.value,
    })
    result.value = { type: 'compress', output }
  } catch (e) {
    error.value = e.toString()
  } finally {
    processing.value = false
  }
}

async function doExtract() {
  processing.value = true
  error.value = null
  result.value = null
  try {
    const outputDir = await open({ directory: true })
    if (!outputDir) {
      processing.value = false
      return
    }
    const files = await invoke('extract_file', {
      archive: filePath.value,
      outputDir,
    })
    result.value = { type: 'extract', files, outputDir }
  } catch (e) {
    error.value = e.toString()
  } finally {
    processing.value = false
  }
}

let unlisten = null

onMounted(async () => {
  unlisten = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'over') {
      dragging.value = true
    } else if (event.payload.type === 'leave') {
      dragging.value = false
    } else if (event.payload.type === 'drop') {
      dragging.value = false
      if (event.payload.paths.length > 0) {
        selectFile(event.payload.paths[0])
      }
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<template>
  <div class="app">
    <header>
      <h1>Collapse</h1>
    </header>

    <div
      class="drop-zone"
      :class="{ active: dragging, 'has-file': filePath }"
      @click="browse"
    >
      <div v-if="!filePath" class="placeholder">
        <div class="drop-icon">+</div>
        <p>Drop a file here or click to browse</p>
      </div>
      <div v-else class="file-info">
        <div class="drop-icon">{{ isArchive ? 'E' : 'C' }}</div>
        <p class="file-name">{{ fileName }}</p>
        <span class="mode-badge">{{ isArchive ? 'Extract' : 'Compress' }}</span>
      </div>
    </div>

    <div v-if="filePath && !result" class="options">
      <template v-if="mode === 'compress'">
        <div class="option-row">
          <div class="field">
            <label>Algorithm</label>
            <select v-model="protocol">
              <option value="zip">ZIP (Deflate)</option>
              <option value="7z">7z (LZMA2)</option>
            </select>
          </div>
          <div class="field">
            <label>Level</label>
            <select v-model.number="level">
              <option v-for="l in 5" :key="l" :value="l">{{ l }}</option>
            </select>
          </div>
        </div>
      </template>

      <button
        class="action-btn"
        :disabled="processing"
        @click="mode === 'compress' ? doCompress() : doExtract()"
      >
        {{ processing ? 'Working...' : (mode === 'compress' ? 'Compress' : 'Extract') }}
      </button>
    </div>

    <div v-if="result" class="result">
      <template v-if="result.type === 'compress'">
        <p class="success-label">Compressed to:</p>
        <p class="output-path">{{ result.output }}</p>
      </template>
      <template v-else>
        <p class="success-label">Extracted {{ result.files.length }} file(s) to:</p>
        <p class="output-path">{{ result.outputDir }}</p>
        <ul class="file-list">
          <li v-for="f in result.files" :key="f">{{ f }}</li>
        </ul>
      </template>
      <button class="reset-btn" @click="reset">New file</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style>
:root {
  --bg: #1a1a2e;
  --surface: #16213e;
  --border: #2a3a5c;
  --text: #e0e0e0;
  --text-muted: #8892a4;
  --accent: #4f8cff;
  --accent-hover: #3a72e0;
  --success: #4caf50;
  --danger: #ef5350;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  -webkit-font-smoothing: antialiased;
  user-select: none;
}

.app {
  display: flex;
  flex-direction: column;
  padding: 24px;
  gap: 20px;
  min-height: 100vh;
}

header h1 {
  font-size: 1.3rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.drop-zone {
  flex: 1;
  min-height: 180px;
  border: 2px dashed var(--border);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.drop-zone:hover,
.drop-zone.active {
  border-color: var(--accent);
  background: rgba(79, 140, 255, 0.06);
}

.drop-zone.has-file {
  border-style: solid;
  border-color: var(--accent);
  background: rgba(79, 140, 255, 0.04);
}

.placeholder,
.file-info {
  text-align: center;
}

.drop-icon {
  width: 48px;
  height: 48px;
  margin: 0 auto 12px;
  border-radius: 50%;
  background: var(--surface);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--accent);
}

.placeholder p {
  color: var(--text-muted);
  font-size: 0.9rem;
}

.file-name {
  font-weight: 600;
  font-size: 1rem;
  margin-bottom: 8px;
  word-break: break-all;
}

.mode-badge {
  display: inline-block;
  font-size: 0.72rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 3px 10px;
  border-radius: 10px;
  background: rgba(79, 140, 255, 0.15);
  color: var(--accent);
}

.options {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.option-row {
  display: flex;
  gap: 12px;
}

.field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field label {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.field select {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text);
  font-size: 0.9rem;
  outline: none;
}

.field select:focus {
  border-color: var(--accent);
}

.action-btn {
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: #fff;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}

.action-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.result {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
}

.success-label {
  font-size: 0.85rem;
  color: var(--success);
  font-weight: 600;
  margin-bottom: 6px;
}

.output-path {
  font-size: 0.82rem;
  color: var(--text-muted);
  word-break: break-all;
  margin-bottom: 10px;
}

.file-list {
  list-style: none;
  margin-bottom: 12px;
  max-height: 120px;
  overflow-y: auto;
}

.file-list li {
  font-size: 0.82rem;
  color: var(--text-muted);
  padding: 3px 0;
}

.reset-btn {
  width: 100%;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: transparent;
  color: var(--text);
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.reset-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.error {
  text-align: center;
  color: var(--danger);
  font-size: 0.85rem;
}
</style>
