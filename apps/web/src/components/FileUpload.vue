<script setup>
import { ref } from 'vue'
import { uploadFile } from '../services/api.js'

const emit = defineEmits(['uploaded'])

const file = ref(null)
const algorithm = ref('7z')
const level = ref(5)
const dragging = ref(false)
const uploading = ref(false)
const error = ref(null)

function onFileChange(e) {
  file.value = e.target.files[0] || null
}

function onDrop(e) {
  dragging.value = false
  const dropped = e.dataTransfer.files[0]
  if (dropped) file.value = dropped
}

async function submit() {
  if (!file.value) return
  uploading.value = true
  error.value = null
  try {
    await uploadFile(file.value, algorithm.value, level.value)
    file.value = null
    emit('uploaded')
  } catch (e) {
    error.value = e.message
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <section class="upload-card">
    <h2>Upload File</h2>

    <div
      class="dropzone"
      :class="{ active: dragging, 'has-file': file }"
      @dragover.prevent="dragging = true"
      @dragleave="dragging = false"
      @drop.prevent="onDrop"
      @click="$refs.fileInput.click()"
    >
      <input
        ref="fileInput"
        type="file"
        hidden
        @change="onFileChange"
      />
      <div v-if="file" class="file-info">
        <span class="icon">📄</span>
        <span class="name">{{ file.name }}</span>
        <span class="size">{{ (file.size / 1024).toFixed(1) }} KB</span>
      </div>
      <div v-else class="placeholder">
        <span class="icon">⬆</span>
        <p>Drop a file here or click to browse</p>
      </div>
    </div>

    <div class="options">
      <div class="field">
        <label>Algorithm</label>
        <select v-model="algorithm">
          <option value="7z">7z (LZMA2)</option>
          <option value="zip">ZIP (Deflate)</option>
        </select>
      </div>
      <div class="field">
        <label>Level</label>
        <select v-model.number="level">
          <option v-for="l in 5" :key="l" :value="l">{{ l }}</option>
        </select>
      </div>
    </div>

    <button
      class="btn primary"
      :disabled="!file || uploading"
      @click="submit"
    >
      {{ uploading ? 'Uploading...' : 'Compress' }}
    </button>

    <p v-if="error" class="error">{{ error }}</p>
  </section>
</template>

<style scoped>
.upload-card {
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
}

h2 {
  margin: 0 0 16px;
  font-size: 1.15rem;
  font-weight: 600;
}

.dropzone {
  border: 2px dashed var(--border);
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
}
.dropzone:hover,
.dropzone.active {
  border-color: var(--accent);
  background: var(--accent-bg);
}
.dropzone.has-file {
  border-style: solid;
  border-color: var(--accent);
}

.placeholder .icon {
  font-size: 1.8rem;
  display: block;
  margin-bottom: 8px;
}
.placeholder p {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.9rem;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: center;
}
.file-info .icon { font-size: 1.4rem; }
.file-info .name { font-weight: 500; }
.file-info .size { color: var(--text-muted); font-size: 0.85rem; }

.options {
  display: flex;
  gap: 16px;
  margin: 16px 0;
}

.field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.field label {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.field select {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--text);
  font-size: 0.9rem;
}

.btn {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}
.btn.primary {
  background: var(--accent);
  color: #fff;
}
.btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error {
  margin: 10px 0 0;
  color: var(--danger);
  font-size: 0.85rem;
}
</style>
