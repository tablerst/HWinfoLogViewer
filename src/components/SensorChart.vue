<!-- SensorChart.vue -->
<template>
  <div ref="chartRef" class="chart"></div>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, ref, watch} from 'vue';
import * as echarts from 'echarts/core';
import {GridComponent, GridComponentOption} from 'echarts/components';
import {LineChart, LineSeriesOption} from 'echarts/charts';
import {UniversalTransition} from 'echarts/features';
import {CanvasRenderer} from 'echarts/renderers';
import {useRoute} from 'vue-router'
import {invoke} from '@tauri-apps/api/core';

echarts.use([GridComponent, LineChart, CanvasRenderer, UniversalTransition]);

type EChartsOption = echarts.ComposeOption<
    GridComponentOption | LineSeriesOption
>;

const route = useRoute();
const fieldKey = route.params.fieldKey as string;
const chartRef = ref<HTMLElement | null>(null);
let chartInstance: echarts.ECharts | null = null;

const extractFormattedName = (key: string) => key.split('-').pop()!.replace(/_/g, ' ');

const getData = async (rawKey: string): Promise<any[]> => {
  try {
    const res = await invoke<string>('get_data_by_key', {key: extractFormattedName(rawKey)});
    const data = JSON.parse(res);
    console.log('获取的数据:', data);
    return data as any[];
  } catch (err) {
    console.error('拉取传感器数据失败', err);
    return [];
  }
};

const updateChart = (data: any[]) => {
  if (!chartInstance) return;

  const times = data.map(item => item.Time);
  const values = data.map(item => {
    const field = Object.keys(item).find(k => k !== 'Time' && k !== 'Date')!;
    return item[field];
  });

  chartInstance.setOption({
    xAxis: {type: 'category', data: times},
    yAxis: {type: 'value'},
    series: [{data: values, type: 'line'}]
  } as EChartsOption);
};


onMounted(async () => {
  if (!chartRef.value) return;
  chartInstance = echarts.init(chartRef.value);
  window.addEventListener('resize', () => chartInstance?.resize());

  const initKey = fieldKey.toString();
  const initData = await getData(initKey);
  updateChart(initData)
});

watch(
    () => route.params.fieldKey,
    async (newKey, oldKey) => {
      console.log('fieldKey 更新为:', newKey)
      if (newKey && newKey !== oldKey) {
        const newData = await getData(newKey as string);
        updateChart(newData);
      }
    },
    {immediate: false}
)

onUnmounted(() => {
  window.removeEventListener('resize', () => chartInstance?.resize());
  chartInstance?.dispose();
});
</script>

<style scoped>
.chart {
  width: 100%;
  height: 60vh;
}
</style>