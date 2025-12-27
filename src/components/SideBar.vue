<template>
  <n-menu
      v-model:value="activeKey"
      :collapsed-width="64"
      :collapsed-icon-size="22"
      :options="menuOptions"
      :root-indent="16"
      :indent="24"
      class="sidebar-menu"
      @update:value="handleMenuSelect"
  />
</template>

<script setup lang="ts">
import {Component, h, onMounted, ref} from 'vue';
import {useRouter} from 'vue-router';
import {NIcon, useMessage} from 'naive-ui';
import {useI18n} from 'vue-i18n';
import {BarChartOutline, HardwareChipOutline, HomeOutline, SettingsOutline} from '@vicons/ionicons5';
import {invoke} from '@tauri-apps/api/core';
import {emitter} from "../utils/eventBus.ts";
import {formatError} from '../utils/formatError'
import {parseSensorLabel} from '../utils/sensorLabel'

interface MenuOption {
  label: string | (() => any);
  key: string;
  icon?: Component;
  children?: MenuOption[];
}

function renderIcon(icon: Component) {
  return () => h(NIcon, null, {default: () => h(icon)});
}

const activeKey = ref('home');
const router = useRouter();
const message = useMessage();
const {t} = useI18n();
const menuOptions = ref<MenuOption[]>([
  {
    label: () => t('nav.home'),
    key: 'home',
    icon: renderIcon(HomeOutline)
  },
  {
    label: () => t('nav.sensorData'),
    key: 'sensor-data',
    icon: renderIcon(BarChartOutline),
    children: []
  },
  {
    label: () => t('nav.settings'),
    key: 'settings',
    icon: renderIcon(SettingsOutline)
  }
])


function handleMenuSelect(key: string) {
  if (key === 'home') {
    router.push({name: 'Home'});
  } else if (key === 'settings') {
    router.push({name: 'Settings'});
  } else if (key.startsWith('sensor-')) {
    router.push({name: 'SensorDetail', params: {fieldKey: key}});
  }
}

async function getLogData() {
  const pending = message.loading(t('sidebar.loadingData'), {duration: 0});
  try {
    const raw = await invoke<string>('get_data');
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch (err) {
      throw new Error(t('sidebar.parseBackendFailed', { error: formatError(err, t('common.unknownError')) }));
    }

    console.log('获取的数据:', parsed);

    if (Array.isArray(parsed) && parsed.length === 0) {
      // 清空传感器菜单
      menuOptions.value = menuOptions.value.map(opt =>
          opt.key === 'sensor-data' ? {...opt, children: []} : opt
      );
      pending.destroy();
      message.info(t('sidebar.noDataHint'));
      return;
    }

    const tree = Array.isArray(parsed) ? parsed[0] : (parsed as Record<string, any>);
    const sensorChildren = convertToMenuOptions(tree);
    menuOptions.value = menuOptions.value.map(opt =>
        opt.key === 'sensor-data' ? {...opt, children: sensorChildren} : opt
    );
    pending.destroy();
    message.success(t('sidebar.dataLoaded'));
  } catch (err) {
    console.error('获取数据失败:', err);
    pending.destroy();
    message.error(t('sidebar.dataLoadFailed', { error: formatError(err, t('common.unknownError')) }));
  }
}

function convertToMenuOptions(
    obj: Record<string, any>,
    parentKey = "sensor"
): MenuOption[] {
  return Object.entries(obj)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, value]) => {
        const safeKey = `${parentKey}-${name}`.replace(/\s+/g, "_");
        const option: MenuOption = {
          label: name,
          key: safeKey,
          icon: renderIcon(HardwareChipOutline)
          // TODO: support custom icon
        };
        if (value && typeof value === "object") {
          const leaves: MenuOption[] = Object.entries(value)
              .filter(([k, v]) => k !== 'children' && typeof v !== 'object')
              .map(([subName]) => {
                const raw = String(subName);
                const meta = parseSensorLabel(raw);
                const displayName = meta.baseName || raw;
                const unit = meta.unit;
                return {
                  label: () => h(
                    'span',
                    {
                      class: 'menu-label',
                      title: raw
                    },
                    [
                      h('span', {class: 'menu-label__name'}, displayName),
                      unit ? h('span', {class: 'menu-label__unit'}, `[${unit}]`) : null
                    ]
                  ),
                  // Key must keep raw field name for backend exact match.
                  key: `${safeKey}-${raw}`.replace(/\s+/g, '_'),
                  icon: renderIcon(HardwareChipOutline)
                };
              });

          if (value.children) {
            leaves.push(...convertToMenuOptions(value.children, safeKey));
          }
          if (leaves.length) option.children = leaves;
        }
        return option;
      });
}


onMounted(() => {
  void getLogData();
})

emitter.on('data-loaded', () => {
  void getLogData();
})
</script>

<style scoped>
.sidebar-menu {
  min-width: 200px;
  max-height: 100vh;
  overflow-y: auto;
}

.sidebar-menu :deep(.n-menu-item-content) {
  display: flex;
  align-items: center;
}

.sidebar-menu :deep(.n-menu-item-content-header) {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-menu :deep(.n-menu-item-content-header[title]) {
  cursor: help;
}

.menu-label {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}

.menu-label__name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.menu-label__unit {
  color: #80868b;
  font-size: 12px;
  white-space: nowrap;
}
</style>