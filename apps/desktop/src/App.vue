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
    <div class="bg-glow"></div>

    <header>
      <div class="logo-row">
        <svg class="logo" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="g1" x1="0" y1="0" x2="40" y2="40" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#a78bfa"/>
              <stop offset="100%" stop-color="#6d28d9"/>
            </linearGradient>
            <linearGradient id="g2" x1="10" y1="5" x2="35" y2="38" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#c4b5fd" stop-opacity="0.6"/>
              <stop offset="100%" stop-color="#7c3aed" stop-opacity="0.2"/>
            </linearGradient>
          </defs>
          <!-- Abstract collapsing cube -->
          <path d="M20 4L36 13V27L20 36L4 27V13L20 4Z" fill="url(#g1)" opacity="0.9"/>
          <path d="M20 4L36 13L20 22L4 13L20 4Z" fill="url(#g2)"/>
          <path d="M20 22V36L4 27V13L20 22Z" fill="#7c3aed" opacity="0.5"/>
          <path d="M20 22V36L36 27V13L20 22Z" fill="#6d28d9" opacity="0.35"/>
          <!-- Inner collapse lines -->
          <path d="M20 14L28 18.5L20 23L12 18.5L20 14Z" fill="white" opacity="0.12"/>
          <line x1="20" y1="23" x2="20" y2="30" stroke="white" stroke-opacity="0.1" stroke-width="1"/>
        </svg>
        <div class="brand">
          <h1>Collapse</h1>
          <span class="version">v0.1.0</span>
        </div>
      </div>
    </header>

    <main>
      <!-- Drop zone -->
      <div
        class="drop-zone"
        :class="{ active: dragging, 'has-file': filePath }"
        @click="browse"
      >
        <Transition name="fade" mode="out-in">
          <div v-if="!filePath" class="placeholder" key="empty">
            <div class="drop-ring">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M12 5v14M5 12h14"/>
              </svg>
            </div>
            <p class="drop-label">Drop a file here</p>
            <p class="drop-hint">or click to browse</p>
          </div>
          <div v-else class="file-selected" key="file">
            <div class="file-icon-wrap" :class="mode">
              <svg v-if="mode === 'compress'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 3v18M3 12l9 9 9-9"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 21V3M3 12l9-9 9 9"/>
              </svg>
            </div>
            <p class="file-name">{{ fileName }}</p>
            <span class="mode-pill" :class="mode">
              {{ mode === 'compress' ? 'Compress' : 'Extract' }}
            </span>
          </div>
        </Transition>
      </div>

      <!-- Options -->
      <Transition name="slide">
        <div v-if="filePath && !result" class="controls">
          <template v-if="mode === 'compress'">
            <div class="option-row">
              <div class="field">
                <label>Format</label>
                <div class="select-wrap">
                  <select v-model="protocol">
                    <option value="zip">ZIP</option>
                    <option value="7z">7z</option>
                  </select>
                </div>
              </div>
              <div class="field">
                <label>Level</label>
                <div class="level-bar">
                  <button
                    v-for="l in 5" :key="l"
                    :class="{ active: level === l }"
                    @click="level = l"
                  >{{ l }}</button>
                </div>
              </div>
            </div>
          </template>

          <button
            class="action-btn"
            :class="{ working: processing }"
            :disabled="processing"
            @click="mode === 'compress' ? doCompress() : doExtract()"
          >
            <span v-if="processing" class="spinner"></span>
            {{ processing ? 'Working...' : (mode === 'compress' ? 'Compress' : 'Extract') }}
          </button>
        </div>
      </Transition>

      <!-- Result -->
      <Transition name="slide">
        <div v-if="result" class="result-card">
          <div class="result-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 6L9 17l-5-5"/>
            </svg>
          </div>
          <template v-if="result.type === 'compress'">
            <p class="result-title">Compressed successfully</p>
            <p class="result-path">{{ result.output }}</p>
          </template>
          <template v-else>
            <p class="result-title">Extracted {{ result.files.length }} file(s)</p>
            <p class="result-path">{{ result.outputDir }}</p>
            <ul class="extracted-list">
              <li v-for="f in result.files" :key="f">{{ f }}</li>
            </ul>
          </template>
          <button class="ghost-btn" @click="reset">Start over</button>
        </div>
      </Transition>

      <!-- Error -->
      <Transition name="fade">
        <div v-if="error" class="error-bar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
          <span>{{ error }}</span>
        </div>
      </Transition>
    </main>
  </div>
</template>

<style>
:root {
  --bg: #0c0a13;
  --surface: rgba(255, 255, 255, 0.04);
  --surface-hover: rgba(255, 255, 255, 0.07);
  --border: rgba(255, 255, 255, 0.08);
  --border-hover: rgba(255, 255, 255, 0.14);
  --text: #f0eef5;
  --text-secondary: rgba(240, 238, 245, 0.55);
  --text-tertiary: rgba(240, 238, 245, 0.3);
  --accent: #a78bfa;
  --accent-dim: rgba(167, 139, 250, 0.15);
  --accent-solid: #7c3aed;
  --success: #34d399;
  --success-dim: rgba(52, 211, 153, 0.12);
  --danger: #f87171;
  --danger-dim: rgba(248, 113, 113, 0.1);
  --radius: 14px;
  --radius-sm: 10px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', system-ui, sans-serif;
  background: var(--bg);
  color: var(--text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  user-select: none;
  overflow: hidden;
}

.app {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

/* Ambient glow */
.bg-glow {
  position: fixed;
  top: -40%;
  left: 50%;
  transform: translateX(-50%);
  width: 600px;
  height: 600px;
  background: radial-gradient(circle, rgba(124, 58, 237, 0.12) 0%, transparent 70%);
  pointer-events: none;
  z-index: 0;
}

/* Header */
header {
  position: relative;
  z-index: 1;
  padding: 20px 24px 0;
}

.logo-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo {
  width: 36px;
  height: 36px;
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.brand h1 {
  font-size: 1.15rem;
  font-weight: 650;
  letter-spacing: -0.025em;
  background: linear-gradient(135deg, #f0eef5 30%, var(--accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.version {
  font-size: 0.7rem;
  color: var(--text-tertiary);
  font-weight: 500;
}

/* Main */
main {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px 24px 24px;
  gap: 14px;
}

/* Drop zone */
.drop-zone {
  flex: 1;
  min-height: 200px;
  border: 1.5px dashed var(--border);
  border-radius: var(--radius);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: var(--surface);
  backdrop-filter: blur(20px);
}

.drop-zone:hover {
  border-color: var(--border-hover);
  background: var(--surface-hover);
}

.drop-zone.active {
  border-color: var(--accent);
  background: var(--accent-dim);
  border-style: solid;
}

.drop-zone.has-file {
  border-style: solid;
  border-color: var(--border-hover);
  background: var(--surface);
}

.placeholder, .file-selected {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.drop-ring {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  border: 1.5px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: all 0.3s;
}

.drop-ring svg {
  width: 22px;
  height: 22px;
}

.drop-zone:hover .drop-ring,
.drop-zone.active .drop-ring {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-dim);
}

.drop-label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.drop-hint {
  font-size: 0.78rem;
  color: var(--text-tertiary);
}

/* File selected */
.file-icon-wrap {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-dim);
  color: var(--accent);
}

.file-icon-wrap.extract {
  background: var(--success-dim);
  color: var(--success);
}

.file-icon-wrap svg {
  width: 22px;
  height: 22px;
}

.file-name {
  font-weight: 600;
  font-size: 0.95rem;
  word-break: break-all;
  max-width: 100%;
  line-height: 1.3;
}

.mode-pill {
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding: 4px 12px;
  border-radius: 20px;
  background: var(--accent-dim);
  color: var(--accent);
}

.mode-pill.extract {
  background: var(--success-dim);
  color: var(--success);
}

/* Controls */
.controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.option-row {
  display: flex;
  gap: 10px;
}

.field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-tertiary);
}

.select-wrap {
  position: relative;
}

.select-wrap select {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-size: 0.85rem;
  font-weight: 500;
  outline: none;
  appearance: none;
  cursor: pointer;
  transition: border-color 0.2s;
}

.select-wrap select:focus {
  border-color: var(--accent);
}

.level-bar {
  display: flex;
  gap: 4px;
}

.level-bar button {
  flex: 1;
  padding: 10px 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.level-bar button:hover {
  border-color: var(--border-hover);
  color: var(--text);
}

.level-bar button.active {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
}

/* Action button */
.action-btn {
  width: 100%;
  padding: 13px;
  border: none;
  border-radius: var(--radius-sm);
  background: linear-gradient(135deg, var(--accent-solid), #9333ea);
  color: #fff;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  letter-spacing: 0.01em;
}

.action-btn:hover:not(:disabled) {
  filter: brightness(1.15);
  transform: translateY(-1px);
}

.action-btn:active:not(:disabled) {
  transform: translateY(0);
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.25);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Result */
.result-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
  backdrop-filter: blur(20px);
}

.result-icon {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--success-dim);
  color: var(--success);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.result-icon svg {
  width: 22px;
  height: 22px;
}

.result-title {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--success);
}

.result-path {
  font-size: 0.78rem;
  color: var(--text-tertiary);
  word-break: break-all;
  line-height: 1.4;
}

.extracted-list {
  list-style: none;
  max-height: 100px;
  overflow-y: auto;
  width: 100%;
  margin-top: 4px;
}

.extracted-list li {
  font-size: 0.78rem;
  color: var(--text-secondary);
  padding: 3px 0;
  border-bottom: 1px solid var(--border);
}

.extracted-list li:last-child {
  border-bottom: none;
}

.ghost-btn {
  margin-top: 6px;
  padding: 9px 24px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.ghost-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-dim);
}

/* Error */
.error-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-radius: var(--radius-sm);
  background: var(--danger-dim);
  border: 1px solid rgba(248, 113, 113, 0.15);
  color: var(--danger);
  font-size: 0.82rem;
}

.error-bar svg {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

/* Transitions */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.slide-enter-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.slide-leave-active {
  transition: all 0.2s ease;
}
.slide-enter-from {
  opacity: 0;
  transform: translateY(8px);
}
.slide-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}
</style>
