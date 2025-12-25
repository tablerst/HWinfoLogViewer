<template>
  <n-button @click="selectCsv">选择 CSV 文件</n-button>
  <n-button type="primary" :disabled="!filePath" @click="uploadCsv">
    上传并处理
  </n-button>
</template>

<script setup lang="ts">
import {ref} from 'vue'
import {NButton, useMessage} from 'naive-ui'
import {open} from '@tauri-apps/plugin-dialog'
import {invoke} from '@tauri-apps/api/core'
import {emitter} from "../utils/eventBus.ts";
import {formatError} from '../utils/formatError'

const filePath = ref<string | null>(null)
const message = useMessage()

async function selectCsv() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{name: 'CSV 文件', extensions: ['csv']}]
    }) as string | string[] | null

    console.log('Selected file:', selected)
    if (typeof selected === 'string') {
      filePath.value = selected
      message.success(`已选择 CSV：${selected}`)
      return
    }
    if (Array.isArray(selected) && selected.length > 0) {
      filePath.value = selected[0]
      message.success(`已选择 CSV：${selected[0]}`)
      return
    }

    message.info('已取消选择')
  } catch (err) {
    message.error(`选择 CSV 文件失败：${formatError(err)}`)
  }
}

async function uploadCsv() {
  if (!filePath.value) {
    message.warning('请先选择 CSV 文件')
    return
  }

  const pending = message.loading('正在处理 CSV…', {duration: 0})
  try {
    await invoke('load_csv', {path: filePath.value})
    pending.destroy()
    message.success('CSV 处理成功')
    emitter.emit('data-loaded')
  } catch (err) {
    pending.destroy()
    message.error(`CSV 处理失败：${formatError(err)}`)
  }
}

</script>


<style scoped>
</style>
