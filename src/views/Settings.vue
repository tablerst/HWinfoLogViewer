<template>
  <div class="settings-container">
    <n-card :title="t('settings.preferencesTitle')" class="settings-card" style="margin-bottom: 16px">
      <n-space vertical size="large">
        <n-space vertical>
          <n-text depth="3">{{ t('common.language') }}</n-text>
          <n-select
            v-model:value="selectedLocale"
            :options="languageOptions"
            style="max-width: 240px"
          />
        </n-space>
      </n-space>
    </n-card>

    <n-card :title="t('settings.dataManagementTitle')" class="settings-card">
      <n-space vertical size="large">
        <n-space vertical>
          <n-text depth="3">{{ t('settings.currentFilePath') }}</n-text>
          <n-input 
            :value="appStore.filePath || ''" 
            :placeholder="t('settings.noFileSelected')" 
            readonly 
            @click="selectCsv"
          />
        </n-space>

        <n-space>
          <n-button @click="selectCsv">{{ t('settings.chooseNewFile') }}</n-button>
          <n-button 
            type="primary" 
            :disabled="!appStore.filePath" 
            :loading="loading"
            @click="uploadCsv"
          >
            {{ t('settings.reload') }}
          </n-button>
          <n-popconfirm @positive-click="clearData">
            <template #trigger>
              <n-button type="error" ghost :disabled="!appStore.isLoaded">
                {{ t('settings.clearData') }}
              </n-button>
            </template>
            {{ t('settings.clearConfirm') }}
          </n-popconfirm>
        </n-space>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/appStore'
import { emitter } from '../utils/eventBus'
import { formatError } from '../utils/formatError'

import type { AppLocale } from '../utils/locale'

const appStore = useAppStore()
const message = useMessage()
const loading = ref(false)

const { t } = useI18n()

const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' }
]

const selectedLocale = computed({
  get: () => appStore.locale,
  set: (v: AppLocale) => {
    appStore.setLocale(v)
  }
})

async function selectCsv() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: t('common.csvFile'), extensions: ['csv'] }]
    }) as string | string[] | null

    if (typeof selected === 'string') {
      appStore.setFilePath(selected)
      message.success(t('settings.selectCsvSuccess', { path: selected }))
    } else if (Array.isArray(selected) && selected.length > 0) {
      appStore.setFilePath(selected[0])
      message.success(t('settings.selectCsvSuccess', { path: selected[0] }))
    }
  } catch (err) {
    message.error(t('settings.selectCsvFailed', { error: formatError(err, t('common.unknownError')) }))
  }
}

async function uploadCsv() {
  if (!appStore.filePath) {
    message.warning(t('settings.chooseCsvFirst'))
    return
  }

  loading.value = true
  const pending = message.loading(t('settings.processingCsv'), { duration: 0 })

  try {
    await invoke('load_csv', { path: appStore.filePath })
    pending.destroy()
    message.success(t('settings.csvProcessedOk'))
    appStore.setLoaded(true)
    emitter.emit('data-loaded')
  } catch (err) {
    pending.destroy()
    message.error(t('settings.csvProcessFailed', { error: formatError(err, t('common.unknownError')) }))
  } finally {
    loading.value = false
  }
}

function clearData() {
  appStore.clearData()
  message.success(t('settings.dataCleared'))
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
