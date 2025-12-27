import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const filePath = ref<string | null>(null)
  const isLoaded = ref(false)

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

  return {
    filePath,
    isLoaded,
    setFilePath,
    setLoaded,
    clearData
  }
})
