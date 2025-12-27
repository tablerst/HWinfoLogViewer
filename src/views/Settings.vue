<template>
  <div class="settings-container">
    <n-card title="数据管理" class="settings-card">
      <n-space vertical size="large">
        <n-space vertical>
          <n-text depth="3">当前文件路径</n-text>
          <n-input 
            :value="appStore.filePath || ''" 
            placeholder="未选择文件" 
            readonly 
            @click="selectCsv"
          />
        </n-space>

        <n-space>
          <n-button @click="selectCsv">选择新文件</n-button>
          <n-button 
            type="primary" 
            :disabled="!appStore.filePath" 
            :loading="loading"
            @click="uploadCsv"
          >
            重新加载
          </n-button>
          <n-popconfirm @positive-click="clearData">
            <template #trigger>
              <n-button type="error" ghost :disabled="!appStore.isLoaded">
                清除数据
              </n-button>
            </template>
            确定要清除当前加载的数据吗？
          </n-popconfirm>
        </n-space>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useMessage } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/appStore'
import { emitter } from '../utils/eventBus'
import { formatError } from '../utils/formatError'

const appStore = useAppStore()
const message = useMessage()
const loading = ref(false)

async function selectCsv() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'CSV 文件', extensions: ['csv'] }]
    }) as string | string[] | null

    if (typeof selected === 'string') {
      appStore.setFilePath(selected)
      message.success(`已选择 CSV：${selected}`)
    } else if (Array.isArray(selected) && selected.length > 0) {
      appStore.setFilePath(selected[0])
      message.success(`已选择 CSV：${selected[0]}`)
    }
  } catch (err) {
    message.error(`选择 CSV 文件失败：${formatError(err)}`)
  }
}

async function uploadCsv() {
  if (!appStore.filePath) {
    message.warning('请先选择 CSV 文件')
    return
  }

  loading.value = true
  const pending = message.loading('正在处理 CSV…', { duration: 0 })

  try {
    await invoke('load_csv', { path: appStore.filePath })
    pending.destroy()
    message.success('CSV 处理成功')
    appStore.setLoaded(true)
    emitter.emit('data-loaded')
  } catch (err) {
    pending.destroy()
    message.error(`CSV 处理失败：${formatError(err)}`)
  } finally {
    loading.value = false
  }
}

function clearData() {
  appStore.clearData()
  message.success('数据已清除')
  // Trigger sidebar refresh (empty)
  emitter.emit('data-loaded') 
}
</script>

<style scoped>
.settings-container {
  max-width: 800px;
  margin: 0 auto;
}
</style>
