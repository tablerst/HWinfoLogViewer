<!-- SensorChart.vue -->
<template>
  <div>
    <n-card title="数据分析" size="small" bordered>
      <n-row :gutter="16">
        <n-col :span="6">
          <n-statistic
              label="最小值"
              :value="stats.min"
              tabular-nums
              :value-style="{ color: '#1E90FF' }"
          />
        </n-col>
        <n-col :span="6">
          <n-statistic
              label="最大值"
              :value="stats.max"
              tabular-nums
              :value-style="{ color: '#FF4500' }"
          />
        </n-col>
        <n-col :span="6">
          <n-statistic
              label="平均值"
              :value="stats.avg.toFixed(2)"
              tabular-nums
              :value-style="{ color: '#008000' }"
          />
        </n-col>
        <n-col :span="6">
          <n-statistic
              label="中位数"
              :value="stats.median"
              tabular-nums
              :value-style="{ color: '#FFA500' }"
          />
        </n-col>
      </n-row>
    </n-card>

    <div ref="chartRef" class="chart"></div>
  </div>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref, watch} from 'vue';
import * as echarts from 'echarts/core';
import {
  DataZoomComponent,
  DataZoomComponentOption,
  DataZoomInsideComponent,
  GridComponent,
  GridComponentOption,
  LegendComponent,
  LegendComponentOption,
  MarkLineComponent,
  MarkLineComponentOption,
  MarkPointComponent,
  MarkPointComponentOption,
  ToolboxComponent,
  ToolboxComponentOption,
  TooltipComponent,
  TooltipComponentOption
} from 'echarts/components';
import {LineChart, LineSeriesOption} from 'echarts/charts';
import {UniversalTransition} from 'echarts/features';
import {CanvasRenderer} from 'echarts/renderers';
import {useRoute} from 'vue-router'
import {invoke} from '@tauri-apps/api/core';
import {NCard, NCol, NRow, NStatistic, useMessage} from 'naive-ui';
import {formatError} from '../utils/formatError'

echarts.use([
  TooltipComponent,
  LegendComponent,
  GridComponent,
  LineChart,
  CanvasRenderer,
  UniversalTransition,
  DataZoomComponent,
  DataZoomInsideComponent,
  ToolboxComponent,
  MarkLineComponent,
  MarkPointComponent,
]);

type EChartsOption = echarts.ComposeOption<
    GridComponentOption
    | LineSeriesOption
    | TooltipComponentOption
    | LegendComponentOption
    | DataZoomComponentOption
    | ToolboxComponentOption
    | MarkLineComponentOption
    | MarkPointComponentOption
>;

const route = useRoute();
const fieldKey = route.params.fieldKey as string;
const chartRef = ref<HTMLElement | null>(null);
let chartInstance: echarts.ECharts | null = null;
const message = useMessage();

// Data for card
const stats = reactive({
  min: 0,
  max: 0,
  avg: 0,
  median: 0,
  current: 0
});

const extractFormattedName = (key: string) => key.split('-').pop()!.replace(/_/g, ' ');

const getData = async (rawKey: string): Promise<any[]> => {
  const formattedName = extractFormattedName(rawKey);
  const pending = message.loading(`正在加载「${formattedName}」数据…`, {duration: 0});
  try {
    const res = await invoke<string>('get_data_by_key', {key: formattedName});
    let data: unknown;
    try {
      data = JSON.parse(res);
    } catch (err) {
      pending.destroy();
      message.error(`解析「${formattedName}」数据失败：${formatError(err)}`);
      return [];
    }
    console.log('获取的数据:', data);

    pending.destroy();
    const arr = Array.isArray(data) ? (data as any[]) : [];
    message.success(`已加载「${formattedName}」数据（${arr.length} 条）`);
    return arr;
  } catch (err) {
    pending.destroy();
    console.error('拉取传感器数据失败', err);
    message.error(`加载「${formattedName}」数据失败：${formatError(err)}`);
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

  if (values.length) {
    const solvedValues = values.map(v => Number(v)).filter(v => !isNaN(v));

    stats.min = Math.min(...solvedValues);
    stats.max = Math.max(...solvedValues);
    stats.avg = solvedValues.reduce((a, b) => a + b, 0) / solvedValues.length;

    const sorted = [...solvedValues].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    stats.median = sorted.length % 2 === 0
        ? (sorted[mid - 1] + sorted[mid]) / 2
        : sorted[mid];

    stats.current = solvedValues[solvedValues.length - 1];
  } else {
    Object.assign(stats, {min: 0, max: 0, avg: 0, median: 0, current: 0});
  }

  chartInstance.setOption({
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        label: {backgroundColor: '#555'}
      }
    },
    legend: {
      show: true,
      selectedMode: 'multiple'
    },
    dataZoom: [
      {type: 'slider', start: 0, end: 100},
      {type: 'inside', start: 0, end: 100}
    ],
    toolbox: {
      feature: {
        right: 10,
        saveAsImage: {
          icon: 'path://M16.59 9H15V4c0-.55-.45-1-1-1h-4c-.55 0-1 .45-1 1v5H7.41c-.89 0-1.34 1.08-.71 1.71l4.59 4.59c.39.39 1.02.39 1.41 0l4.59-4.59c.63-.63.19-1.71-.7-1.71zM5 19c0 .55.45 1 1 1h12c.55 0 1-.45 1-1s-.45-1-1-1H6c-.55 0-1 .45-1 1z'
        },
        dataView: {
          readOnly: false,
          icon: 'path://M4 7v2c0 .55-.45 1-1 1s-1 .45-1 1v2c0 .55.45 1 1 1s1 .45 1 1v2c0 1.66 1.34 3 3 3h2c.55 0 1-.45 1-1s-.45-1-1-1H7c-.55 0-1-.45-1-1v-2c0-1.3-.84-2.42-2-2.83v-.34C5.16 11.42 6 10.3 6 9V7c0-.55.45-1 1-1h2c.55 0 1-.45 1-1s-.45-1-1-1H7C5.34 4 4 5.34 4 7zm17 3c-.55 0-1-.45-1-1V7c0-1.66-1.34-3-3-3h-2c-.55 0-1 .45-1 1s.45 1 1 1h2c.55 0 1 .45 1 1v2c0 1.3.84 2.42 2 2.83v.34c-1.16.41-2 1.52-2 2.83v2c0 .55-.45 1-1 1h-2c-.55 0-1 .45-1 1s.45 1 1 1h2c1.66 0 3-1.34 3-3v-2c0-.55.45-1 1-1s1-.45 1-1v-2c0-.55-.45-1-1-1z'
        },
        restore: {
          icon: 'path://M17.65 6.35a7.95 7.95 0 0 0-6.48-2.31c-3.67.37-6.69 3.35-7.1 7.02C3.52 15.91 7.27 20 12 20a7.98 7.98 0 0 0 7.21-4.56c.32-.67-.16-1.44-.9-1.44c-.37 0-.72.2-.88.53a5.994 5.994 0 0 1-6.8 3.31c-2.22-.49-4.01-2.3-4.48-4.52A6.002 6.002 0 0 1 12 6c1.66 0 3.14.69 4.22 1.78l-1.51 1.51c-.63.63-.19 1.71.7 1.71H19c.55 0 1-.45 1-1V6.41c0-.89-1.08-1.34-1.71-.71l-.64.65z'
        }
      }
    },
    xAxis: {type: 'category', data: times},
    yAxis: {type: 'value'},
    series: [{
      data: values,
      type: 'line',
      smooth: true,
      sampling: 'lttb',
      markLine: {
        data: [
          //   TODO: Need to be dynamic
          {yAxis: 2000, name: '下限'},
          {yAxis: 4500, name: '上限'}
        ],
        lineStyle: {type: 'dashed'}
      },
      markPoint: {
        data: [
          {type: 'max', name: '最大值'},
          {type: 'min', name: '最小值'}
        ]
      }
    }]
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
  margin-top: 16px;
}
</style>