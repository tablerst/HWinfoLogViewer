<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/appStore'
import { emitter } from '../utils/eventBus'
import { formatError } from '../utils/formatError'

const appStore = useAppStore()
const router = useRouter()
const message = useMessage()
const loading = ref(false)

const { t } = useI18n()

async function selectCsv() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: t('common.csvFile'), extensions: ['csv'] }]
    }) as string | string[] | null

    if (typeof selected === 'string') {
      appStore.setFilePath(selected)
      message.success(t('dashboard.selectCsvSuccess', { path: selected }))
    } else if (Array.isArray(selected) && selected.length > 0) {
      appStore.setFilePath(selected[0])
      message.success(t('dashboard.selectCsvSuccess', { path: selected[0] }))
    }
  } catch (err) {
    message.error(t('dashboard.selectCsvFailed', { error: formatError(err, t('common.unknownError')) }))
  }
}

async function uploadCsv() {
  if (!appStore.filePath) {
    message.warning(t('dashboard.chooseCsvFirst'))
    return
  }

  loading.value = true
  const pending = message.loading(t('dashboard.processingCsv'), { duration: 0 })

  try {
    await invoke('load_csv', { path: appStore.filePath })
    pending.destroy()
    message.success(t('dashboard.csvProcessDone'))
    appStore.setLoaded(true)
    emitter.emit('data-loaded')
  } catch (err) {
    pending.destroy()
    message.error(t('dashboard.csvProcessFailed', { error: formatError(err, t('common.unknownError')) }))
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
        :title="t('dashboard.welcomeTitle')"
        :description="t('dashboard.welcomeDescription')"
      >
        <template #footer>
          <n-space vertical align="center" :size="24">
            <n-space>
              <n-button @click="selectCsv">{{ t('dashboard.chooseCsv') }}</n-button>
              <n-button 
                type="primary" 
                :disabled="!appStore.filePath" 
                :loading="loading"
                @click="uploadCsv"
              >
                {{ t('dashboard.uploadAndProcess') }}
              </n-button>
            </n-space>
            <n-text v-if="appStore.filePath" depth="3" class="file-path">
              {{ t('dashboard.selectedFilePrefix', { path: appStore.filePath }) }}
            </n-text>
          </n-space>
        </template>
      </n-result>
    </n-card>

    <n-card v-else :title="t('dashboard.dataOverviewTitle')">
      <n-result status="success" :title="t('dashboard.dataLoadedTitle')">
        <template #footer>
          <n-space justify="center">
            <n-button @click="goToSettings">{{ t('dashboard.manageData') }}</n-button>
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
