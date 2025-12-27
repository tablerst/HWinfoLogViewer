import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type SamplingMode = 'auto' | 'none' | 'lttb' | 'average' | 'max' | 'min'
export type YAxisScaleType = 'linear' | 'log'

const STORAGE_KEY = 'hwinfo-log-viewer:chart-prefs:v1'

function safeParsePrefs(raw: string | null): any {
  if (!raw) return null
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

export const useChartPrefsStore = defineStore('chartPrefs', () => {
  const smooth = ref(false)
  const showArea = ref(true)
  const connectNulls = ref(false)
  const sampling = ref<SamplingMode>('auto')
  const yAxisScale = ref<YAxisScaleType>('linear')

  // Load persisted values
  const persisted = safeParsePrefs(localStorage.getItem(STORAGE_KEY))
  if (persisted && typeof persisted === 'object') {
    if (typeof persisted.smooth === 'boolean') smooth.value = persisted.smooth
    if (typeof persisted.showArea === 'boolean') showArea.value = persisted.showArea
    if (typeof persisted.connectNulls === 'boolean') connectNulls.value = persisted.connectNulls
    if (typeof persisted.sampling === 'string') sampling.value = persisted.sampling
    if (typeof persisted.yAxisScale === 'string') yAxisScale.value = persisted.yAxisScale
  }

  watch(
    () => ({
      smooth: smooth.value,
      showArea: showArea.value,
      connectNulls: connectNulls.value,
      sampling: sampling.value,
      yAxisScale: yAxisScale.value
    }),
    (v) => {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(v))
    },
    { deep: true }
  )

  function reset() {
    smooth.value = false
    showArea.value = true
    connectNulls.value = false
    sampling.value = 'auto'
    yAxisScale.value = 'linear'
  }

  return {
    smooth,
    showArea,
    connectNulls,
    sampling,
    yAxisScale,
    reset
  }
})
