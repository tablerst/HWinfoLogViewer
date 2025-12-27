<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/appStore'
import { emitter } from '../utils/eventBus'
import { formatError } from '../utils/formatError'

const appStore = useAppStore()
const router = useRouter()
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
    message.success('CSV 处理完成')
    appStore.setLoaded(true)
    emitter.emit('data-loaded')
  } catch (err) {
    pending.destroy()
    message.error(`处理失败：${formatError(err)}`)
  } finally {
    loading.value = false
  }
}

function goToSettings() {
  router.push('/settings')
}
</script>

<template>
  <div class="dashboard-container">
    <n-card v-if="!appStore.isLoaded" class="welcome-card">
      <n-result
        status="info"
        title="欢迎使用 HWInfo Log Viewer"
        description="请选择并上传 CSV 日志文件以开始分析"
      >
        <template #footer>
          <n-space vertical align="center" :size="24">
            <n-space>
              <n-button @click="selectCsv">选择 CSV 文件</n-button>
              <n-button 
                type="primary" 
                :disabled="!appStore.filePath" 
                :loading="loading"
                @click="uploadCsv"
              >
                上传并处理
              </n-button>
            </n-space>
            <n-text v-if="appStore.filePath" depth="3" class="file-path">
              已选择: {{ appStore.filePath }}
            </n-text>
          </n-space>
        </template>
      </n-result>
    </n-card>

    <n-card v-else title="数据概览">
      <n-result status="success" title="数据已加载">
        <template #footer>
          <n-space justify="center">
            <n-button @click="goToSettings">管理数据</n-button>
          </n-space>
        </template>
      </n-result>
    </n-card>
  </div>
</template>

<style scoped>
.dashboard-container {
  max-width: 800px;
  margin: 0 auto;
  padding-top: 48px;
}

.welcome-card {
  text-align: center;
}

.file-path {
  word-break: break-all;
}
</style>
