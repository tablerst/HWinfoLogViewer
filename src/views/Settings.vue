<template>
  <n-button @click="selectCsv">选择 CSV 文件</n-button>
  <n-button type="primary" :disabled="!filePath" @click="uploadCsv">
    上传并处理
  </n-button>
</template>

<script setup lang="ts">
import {ref} from 'vue'
import {NButton} from 'naive-ui'
import {open} from '@tauri-apps/plugin-dialog'
import {invoke} from '@tauri-apps/api/core'
import {emitter} from "../utils/eventBus.ts";

const filePath = ref<string | null>(null)

async function selectCsv() {
  const selected = await open({
    multiple: false,
    filters: [{name: 'CSV 文件', extensions: ['csv']}]
  }) as string | string[] | null
  console.log('Selected file:', selected)
  if (typeof selected === 'string') {
    filePath.value = selected
  }
}

async function uploadCsv() {
  if (!filePath.value) return
  await invoke('load_csv', {path: filePath.value})
  emitter.emit('data-loaded')
}

</script>


<style scoped>
</style>
