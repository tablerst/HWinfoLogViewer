<template>
  <n-config-provider :locale="naiveLocale" :date-locale="naiveDateLocale">
    <n-message-provider>
      <router-view/>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NConfigProvider,
  NMessageProvider,
  dateEnUS,
  dateZhCN,
  enUS as naiveEnUS,
  zhCN as naiveZhCN
} from 'naive-ui'

import { useAppStore } from './stores/appStore'

const { locale } = useI18n()
const appStore = useAppStore()

watch(
  () => appStore.locale,
  (v) => {
    if (locale.value !== v) locale.value = v
  },
  { immediate: true }
)

const naiveLocale = computed(() => (locale.value === 'zh-CN' ? naiveZhCN : naiveEnUS))
const naiveDateLocale = computed(() => (locale.value === 'zh-CN' ? dateZhCN : dateEnUS))
</script>