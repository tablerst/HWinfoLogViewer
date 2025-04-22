<template>
  <n-menu
      v-model:value="activeKey"
      :collapsed="collapsed"
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
import {NIcon} from 'naive-ui';
import {BarChartOutline, HardwareChipOutline, HomeOutline, SettingsOutline} from '@vicons/ionicons5';
import {invoke} from '@tauri-apps/api/core';
import {emitter} from "../utils/eventBus.ts";

interface MenuOption {
  label: string;
  key: string;
  icon?: Component;
  children?: MenuOption[];
}

function renderIcon(icon: Component) {
  return () => h(NIcon, null, {default: () => h(icon)});
}

const activeKey = ref('home');
const router = useRouter();
const menuOptions = ref<MenuOption[]>([
  {
    label: '首页',
    key: 'home',
    icon: renderIcon(HomeOutline)
  },
  {
    label: '传感器数据',
    key: 'sensor-data',
    icon: renderIcon(BarChartOutline),
    children: []
  },
  {
    label: '系统设置',
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
  try {
    const raw = await invoke<string>('get_data');
    const parsed = JSON.parse(raw);
    console.log('获取的数据:', parsed);

    const tree = Array.isArray(parsed) ? parsed[0] : parsed;
    const sensorChildren = convertToMenuOptions(tree);
    menuOptions.value = menuOptions.value.map(opt =>
        opt.key === 'sensor-data' ? {...opt, children: sensorChildren} : opt
    );
  } catch (e) {
    console.error('获取数据失败:', e);
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
          const leaves = Object.entries(value)
              .filter(([k, v]) => k !== 'children' && typeof v !== 'object')
              .map(([subName]) => ({
                label: subName,
                key: `${safeKey}-${subName}`.replace(/\s+/g, '_'),
                icon: renderIcon(HardwareChipOutline)
              }));

          if (value.children) {
            leaves.push(...convertToMenuOptions(value.children, safeKey));
          }
          if (leaves.length) option.children = leaves;
        }
        return option;
      });
}


onMounted(async () => {
  const raw = await invoke<string>('get_data')
  const parsed = JSON.parse(raw)
  if (Array.isArray(parsed) && parsed.length > 0) {
    await getLogData()
    console.log('获取数据成功')
  }
})

emitter.on('data-loaded', () => {
  getLogData()
})

const props = defineProps({collapsed: Boolean});
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
</style>