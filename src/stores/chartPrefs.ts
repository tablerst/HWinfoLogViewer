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

  // Threshold warning
  const warnEnabled = ref(false)
  const warnMin = ref<number | null>(null)
  const warnMax = ref<number | null>(null)

  // Load persisted values
  const persisted = safeParsePrefs(localStorage.getItem(STORAGE_KEY))
  if (persisted && typeof persisted === 'object') {
    if (typeof persisted.smooth === 'boolean') smooth.value = persisted.smooth
    if (typeof persisted.showArea === 'boolean') showArea.value = persisted.showArea
    if (typeof persisted.connectNulls === 'boolean') connectNulls.value = persisted.connectNulls
    if (typeof persisted.sampling === 'string') sampling.value = persisted.sampling
    if (typeof persisted.yAxisScale === 'string') yAxisScale.value = persisted.yAxisScale

    if (typeof persisted.warnEnabled === 'boolean') warnEnabled.value = persisted.warnEnabled
    if (persisted.warnMin == null) warnMin.value = null
    else if (typeof persisted.warnMin === 'number' && Number.isFinite(persisted.warnMin)) warnMin.value = persisted.warnMin
    if (persisted.warnMax == null) warnMax.value = null
    else if (typeof persisted.warnMax === 'number' && Number.isFinite(persisted.warnMax)) warnMax.value = persisted.warnMax
  }

  watch(
    () => ({
      smooth: smooth.value,
      showArea: showArea.value,
      connectNulls: connectNulls.value,
      sampling: sampling.value,
      yAxisScale: yAxisScale.value,
      warnEnabled: warnEnabled.value,
      warnMin: warnMin.value,
      warnMax: warnMax.value
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

    warnEnabled.value = false
    warnMin.value = null
    warnMax.value = null
  }

  return {
    smooth,
    showArea,
    connectNulls,
    sampling,
    yAxisScale,
    warnEnabled,
    warnMin,
    warnMax,
    reset
  }
})
