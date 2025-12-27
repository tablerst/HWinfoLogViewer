import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

import type { AppLocale } from '../utils/locale'
import { loadPersistedLocale, persistLocale } from '../utils/locale'

export const useAppStore = defineStore('app', () => {
  const filePath = ref<string | null>(null)
  const isLoaded = ref(false)

  const locale = ref<AppLocale>(loadPersistedLocale('zh-CN'))

  watch(
    () => locale.value,
    (v) => {
      persistLocale(v)
    }
  )

  function setFilePath(path: string | null) {
    filePath.value = path
  }

  function setLoaded(loaded: boolean) {
    isLoaded.value = loaded
  }

  function clearData() {
    filePath.value = null
    isLoaded.value = false
  }

  function setLocale(next: AppLocale) {
    locale.value = next
  }

  return {
    filePath,
    isLoaded,
    locale,
    setFilePath,
    setLoaded,
    setLocale,
    clearData
  }
})
