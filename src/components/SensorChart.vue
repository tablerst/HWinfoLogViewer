<!-- SensorChart.vue -->
<template>
  <div class="sensor-chart-wrapper">
    <n-card :title="analysisTitle" size="small" bordered style="margin-bottom: 16px">
      <n-row :gutter="12">
        <n-col :span="12" :m="6">
          <n-statistic label="最小值" :value="minDisplay" tabular-nums>
            <template #prefix>
              <span style="color: #18a058; font-size: 0.8em">↓</span>
            </template>
          </n-statistic>
        </n-col>
        <n-col :span="12" :m="6">
          <n-statistic label="最大值" :value="maxDisplay" tabular-nums>
            <template #prefix>
              <span style="color: #d03050; font-size: 0.8em">↑</span>
            </template>
          </n-statistic>
        </n-col>
        <n-col :span="12" :m="6">
          <n-statistic label="平均值" :value="avgDisplay" tabular-nums/>
        </n-col>
        <n-col :span="12" :m="6">
          <n-statistic label="中位数" :value="medianDisplay" tabular-nums/>
        </n-col>
      </n-row>
    </n-card>

    <n-card :bordered="true" size="small" title="趋势图" class="chart-card">
      <div class="chart-toolbar">
        <div class="toolbar-row">
          <div class="toolbar-group toolbar-group--grow">
            <n-date-picker
              v-model:value="timeRange"
              type="datetimerange"
              size="small"
              clearable
              :disabled="!hasData"
              :is-date-disabled="isDateDisabled"
              class="toolbar-date"
            />
          </div>

          <div class="toolbar-group toolbar-actions">
            <n-button size="small" :disabled="!hasData" tertiary @click="resetView">重置视图</n-button>
          </div>
        </div>

        <div class="toolbar-row">
          <div class="toolbar-group">
            <span class="toolbar-label">降采样</span>
            <n-select
              v-model:value="chartPrefs.sampling"
              size="small"
              :options="samplingOptions"
              :disabled="!hasData"
              class="toolbar-select"
            />
          </div>

          <div class="toolbar-group">
            <span class="toolbar-label">Y 轴</span>
            <n-select
              v-model:value="chartPrefs.yAxisScale"
              size="small"
              :options="yAxisScaleOptions"
              :disabled="!hasData"
              class="toolbar-select--narrow"
            />
          </div>

          <div class="toolbar-group">
            <n-checkbox v-model:checked="chartPrefs.smooth" size="small" :disabled="!hasData">平滑</n-checkbox>
            <n-checkbox v-model:checked="chartPrefs.showArea" size="small" :disabled="!hasData">面积</n-checkbox>
            <n-checkbox v-model:checked="chartPrefs.connectNulls" size="small" :disabled="!hasData">连线缺失</n-checkbox>
          </div>
        </div>

        <div v-if="hasData" class="chart-meta">
          <span>有效值 <b>{{ displayedCount }}</b> / {{ totalCount }}</span>
          <span class="meta-sep">·</span>
          <span>范围：{{ displayedRangeText }}</span>
          <span v-if="displayedMissingCount > 0" class="meta-sep">·</span>
          <span v-if="displayedMissingCount > 0">缺失值：{{ displayedMissingCount }}{{ chartPrefs.connectNulls ? '（已连线）' : '（断线）' }}</span>
          <span v-if="invalidTimeCount > 0" class="meta-sep">·</span>
          <span v-if="invalidTimeCount > 0">无法解析时间：{{ invalidTimeCount }}</span>
        </div>
      </div>

      <div v-show="hasData" ref="chartRef" class="chart"></div>
      <n-empty v-if="!hasData" description="暂无数据" class="empty-chart">
        <template #extra>
          <span style="color: #999">请选择左侧传感器查看数据</span>
        </template>
      </n-empty>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, onUnmounted, reactive, ref, watch, nextTick} from 'vue';
import * as echarts from 'echarts/core';
import {
  DataZoomComponent,
  DataZoomComponentOption,
  DataZoomInsideComponent,
  GridComponent,
  GridComponentOption,
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
import {NButton, NCard, NCheckbox, NCol, NDatePicker, NEmpty, NRow, NSelect, NStatistic, useMessage} from 'naive-ui';
import {formatError} from '../utils/formatError'
import {parseHwinfoDateTimeToMs, formatDateTimeForTooltip, formatTimeTick} from '../utils/hwinfoDateTime'
import {useChartPrefsStore} from '../stores/chartPrefs'
import {parseSensorLabel, formatValueByUnit, formatValueWithUnit} from '../utils/sensorLabel'

echarts.use([
  TooltipComponent,
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
  | DataZoomComponentOption
  | ToolboxComponentOption
  | MarkLineComponentOption
  | MarkPointComponentOption
> & {
  // ECharts v6 的 axis option 类型在当前入口未直接导出，这里做最小放宽以通过类型检查。
  xAxis?: any;
  yAxis?: any;
};

const route = useRoute();
const chartRef = ref<HTMLElement | null>(null);
let chartInstance: echarts.ECharts | null = null;
const message = useMessage();
const hasData = ref(false);
const chartPrefs = useChartPrefsStore();

type Point = [number, number | null];

const rawData = ref<any[]>([]);
const pointsAll = ref<Point[]>([]);
const sensorFieldName = ref<string>('');
const fullTimeRange = ref<[number, number] | null>(null);
const timeRange = ref<[number, number] | null>(null);
const invalidTimeCount = ref(0);
const missingValueCount = ref(0);
const logFallbackWarned = ref(false);
const zoomSyncLocked = ref(false);
const chartReady = ref(false);

// Data for card
const stats = reactive<{
  min: number | string;
  max: number | string;
  avg: number;
  median: number | string;
  current: number | string;
}>({
  min: '-',
  max: '-',
  avg: 0,
  median: '-',
  current: '-'
});

const sensorMeta = computed(() => parseSensorLabel(sensorFieldName.value || ''));
const sensorDisplayName = computed(() => sensorMeta.value.baseName || sensorFieldName.value || '数值');
const sensorUnit = computed(() => sensorMeta.value.unit);
const analysisTitle = computed(() => {
  const u = sensorUnit.value;
  return u ? `数据分析（单位：[${u}]）` : '数据分析';
});

const minDisplay = computed(() => (typeof stats.min === 'number' ? formatValueByUnit(stats.min, sensorUnit.value) : stats.min));
const maxDisplay = computed(() => (typeof stats.max === 'number' ? formatValueByUnit(stats.max, sensorUnit.value) : stats.max));
const medianDisplay = computed(() => (typeof stats.median === 'number' ? formatValueByUnit(stats.median, sensorUnit.value) : stats.median));
const avgDisplay = computed(() => (displayedCount.value > 0 ? formatValueByUnit(stats.avg, sensorUnit.value) : '-'));

const extractFormattedName = (key: string) => key.split('-').pop()!.replace(/_/g, ' ');

const samplingOptions = [
  {label: '自动', value: 'auto'},
  {label: '关闭', value: 'none'},
  {label: 'LTTB', value: 'lttb'},
  {label: '平均', value: 'average'},
  {label: '最大', value: 'max'},
  {label: '最小', value: 'min'}
];

const yAxisScaleOptions = [
  {label: '线性', value: 'linear'},
  {label: '对数', value: 'log'}
];

function countNumeric(points: Point[]) {
  let c = 0;
  for (const p of points) {
    const v = p[1];
    if (typeof v === 'number' && Number.isFinite(v)) c += 1;
  }
  return c;
}

const totalCount = computed(() => countNumeric(pointsAll.value));

function lowerBoundByTs(points: Point[], ts: number) {
  let lo = 0;
  let hi = points.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (points[mid][0] < ts) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

function upperBoundByTs(points: Point[], ts: number) {
  let lo = 0;
  let hi = points.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (points[mid][0] <= ts) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

function normalizeZoomValue(v: unknown): number | null {
  if (typeof v === 'number' && Number.isFinite(v)) return v;
  if (typeof v === 'string') {
    const n = Number(v);
    if (Number.isFinite(n)) return n;
    // Some browsers do not parse 'YYYY-MM-DD HH:mm:ss' reliably; normalize to ISO-ish.
    const maybeIso = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(?:\.\d{1,3})?$/.test(v)
      ? v.replace(' ', 'T')
      : v;
    const ms = Date.parse(maybeIso);
    if (Number.isFinite(ms)) return ms;
  }
  return null;
}

const zoomRange = computed<[number, number] | null>(() => {
  if (timeRange.value) return timeRange.value;
  if (fullTimeRange.value) return fullTimeRange.value;
  return null;
});

const displayedCount = computed(() => {
  const pts = pointsAll.value;
  if (!pts.length) return 0;
  const range = zoomRange.value;
  if (!range) return countNumeric(pts);
  const [start, end] = range;
  const i0 = lowerBoundByTs(pts, start);
  const i1 = upperBoundByTs(pts, end);
  let c = 0;
  for (let i = i0; i < i1; i++) {
    const v = pts[i][1];
    if (typeof v === 'number' && Number.isFinite(v)) c += 1;
  }
  return c;
});

const displayedMissingCount = computed(() => {
  const pts = pointsAll.value;
  if (!pts.length) return 0;
  const range = zoomRange.value;
  let i0 = 0;
  let i1 = pts.length;
  if (range) {
    i0 = lowerBoundByTs(pts, range[0]);
    i1 = upperBoundByTs(pts, range[1]);
  }
  let c = 0;
  for (let i = i0; i < i1; i++) {
    if (pts[i][1] == null) c += 1;
  }
  return c;
});

const displayedRangeText = computed(() => {
  const range = zoomRange.value;
  if (!range) return '-';
  const [start, end] = range;
  return `${formatDateTimeForTooltip(start)} ~ ${formatDateTimeForTooltip(end)}`;
});

const isDateDisabled = (ts: number) => {
  const full = fullTimeRange.value;
  if (!full) return false;
  const [minTs, maxTs] = full;
  return ts < minTs || ts > maxTs;
};

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

function buildPoints(
  data: any[],
  preferredField?: string
): { points: Point[]; field: string; invalidTime: number; missingValue: number } {
  const first = data.find(Boolean);
  const isMetaKey = (k: string) => k === 'Time' || k === 'Date' || k === 'Timestamp' || k === 'timestamp' || k === 'ts';

  const field = (() => {
    if (preferredField && first && Object.prototype.hasOwnProperty.call(first, preferredField)) {
      return preferredField;
    }
    if (!first) return '';
    return Object.keys(first).find(k => !isMetaKey(k)) ?? '';
  })();

  const points: Point[] = [];
  let invalidTime = 0;
  let missingValue = 0;

  for (const item of data) {
    if (!item || typeof item !== 'object') {
      invalidTime += 1;
      continue;
    }
    const tsRaw = (item as any).Timestamp ?? (item as any).timestamp ?? (item as any).ts;
    const tsFromBackend = Number(tsRaw);
    const ts = Number.isFinite(tsFromBackend)
      ? tsFromBackend
      : parseHwinfoDateTimeToMs((item as any).Date, (item as any).Time);
    if (ts == null) {
      invalidTime += 1;
      continue;
    }
    const raw = field ? (item as any)[field] : undefined;
    const v = Number(raw);
    if (!Number.isFinite(v)) {
      // 保留时间戳，用 null 让折线断开（缺失值），避免“看起来没加载”。
      points.push([ts, null]);
      missingValue += 1;
      continue;
    }
    points.push([ts, v]);
  }

  points.sort((a, b) => a[0] - b[0]);
  return {points, field, invalidTime, missingValue};
}

function resetTimeRange() {
  if (fullTimeRange.value) timeRange.value = [...fullTimeRange.value];
}

function applyZoomRangeToChart(range: [number, number]) {
  if (!chartInstance || !chartReady.value) return;
  zoomSyncLocked.value = true;
  try {
    chartInstance.dispatchAction({
      type: 'dataZoom',
      // 对 time 轴来说 value 通常是 ms 时间戳
      startValue: range[0],
      endValue: range[1]
    } as any);
  } finally {
    zoomSyncLocked.value = false;
  }
}

function resetView() {
  // restore 会把 dataZoom 等交互状态恢复到 setOption 的初始状态
  chartInstance?.dispatchAction({type: 'restore'});
  if (fullTimeRange.value) {
    timeRange.value = [...fullTimeRange.value];
  }
}

const renderChart = async () => {
  if (!chartInstance) return;

  const points = pointsAll.value;
  if (!points.length) {
    hasData.value = false;
    Object.assign(stats, {min: '-', max: '-', avg: 0, median: '-', current: '-'});
    chartInstance.clear();
    return;
  }

  hasData.value = true;
  await nextTick();
  chartInstance.resize();

  // 统计值：基于当前窗口范围，并忽略缺失值（null）
  const range = zoomRange.value;
  let i0 = 0;
  let i1 = points.length;
  if (range) {
    i0 = lowerBoundByTs(points, range[0]);
    i1 = upperBoundByTs(points, range[1]);
  }

  const windowValues: number[] = [];
  for (let i = i0; i < i1; i++) {
    const v = points[i][1];
    if (typeof v === 'number' && Number.isFinite(v)) windowValues.push(v);
  }

  if (!windowValues.length) {
    Object.assign(stats, {min: '-', max: '-', avg: 0, median: '-', current: '-'});
  } else {
    stats.min = Math.min(...windowValues);
    stats.max = Math.max(...windowValues);
    stats.avg = windowValues.reduce((a, b) => a + b, 0) / windowValues.length;

    const sorted = [...windowValues].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    stats.median = sorted.length % 2 === 0
      ? (sorted[mid - 1] + sorted[mid]) / 2
      : sorted[mid];
    stats.current = windowValues[windowValues.length - 1];
  }

  const fullSpanMs = points[points.length - 1][0] - points[0][0];

  const samplingEffective = (() => {
    const mode = chartPrefs.sampling;
    const len = countNumeric(points);
    if (mode === 'auto') return len >= 5000 ? 'lttb' : undefined;
    if (mode === 'none') return undefined;
    return mode;
  })();

  const yAxisType = (() => {
    if (chartPrefs.yAxisScale !== 'log') return 'value' as const;
    // Log axis requires positive values
    const hasNonPositive = points.some(p => typeof p[1] === 'number' && p[1] <= 0);
    if (hasNonPositive) return 'value' as const;
    return 'log' as const;
  })();

  const seriesName = sensorDisplayName.value || '数值';
  const unit = sensorUnit.value;
  const showArea = chartPrefs.showArea;
  const shouldAnimate = countNumeric(points) < 2000;

  const hasNulls = points.some(p => p[1] == null);
  const seriesData = chartPrefs.connectNulls
    ? points.filter(p => typeof p[1] === 'number' && Number.isFinite(p[1]))
    : points;
  const samplingSafe = (hasNulls && !chartPrefs.connectNulls) ? undefined : samplingEffective;

  chartInstance.setOption({
    animation: shouldAnimate,
    grid: {
      left: 48,
      right: 24,
      top: 56,
      bottom: 88,
      containLabel: true
    },
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(255, 255, 255, 0.95)',
      borderColor: '#eee',
      borderWidth: 1,
      textStyle: {color: '#333'},
      axisPointer: {
        type: 'cross',
        label: {backgroundColor: '#666'}
      },
      formatter: (params: any) => {
        const p = Array.isArray(params) ? params[0] : params;
        const value = p?.value;
        const ts = Array.isArray(value) ? value[0] : undefined;
        const v = Array.isArray(value) ? value[1] : undefined;
        const timeText = typeof ts === 'number' ? formatDateTimeForTooltip(ts) : '-';
        const valText = (typeof v === 'number' && Number.isFinite(v))
          ? formatValueWithUnit(v, unit)
          : '-';
        return `${timeText}<br/>${seriesName}：${valText}`;
      }
    },
    dataZoom: [
      {
        type: 'slider',
        xAxisIndex: 0,
        height: 24,
        bottom: 12,
        ...(zoomRange.value
          ? {startValue: zoomRange.value[0], endValue: zoomRange.value[1]}
          : {}),
        borderColor: 'transparent',
        backgroundColor: '#f5f7f9',
        fillerColor: 'rgba(32, 128, 240, 0.1)',
        handleStyle: {color: '#2080f0'},
        brushSelect: false
      },
      {
        type: 'inside',
        xAxisIndex: 0,
        filterMode: 'none'
      }
    ],
    toolbox: {
      right: 12,
      top: 8,
      feature: {
        saveAsImage: {
          icon: 'path://M16.59 9H15V4c0-.55-.45-1-1-1h-4c-.55 0-1 .45-1 1v5H7.41c-.89 0-1.34 1.08-.71 1.71l4.59 4.59c.39.39 1.02.39 1.41 0l4.59-4.59c.63-.63.19-1.71-.7-1.71zM5 19c0 .55.45 1 1 1h12c.55 0 1-.45 1-1s-.45-1-1-1H6c-.55 0-1 .45-1 1z'
        },
        dataZoom: {},
        restore: {
          icon: 'path://M17.65 6.35a7.95 7.95 0 0 0-6.48-2.31c-3.67.37-6.69 3.35-7.1 7.02C3.52 15.91 7.27 20 12 20a7.98 7.98 0 0 0 7.21-4.56c.32-.67-.16-1.44-.9-1.44c-.37 0-.72.2-.88.53a5.994 5.994 0 0 1-6.8 3.31c-2.22-.49-4.01-2.3-4.48-4.52A6.002 6.002 0 0 1 12 6c1.66 0 3.14.69 4.22 1.78l-1.51 1.51c-.63.63-.19 1.71.7 1.71H19c.55 0 1-.45 1-1V6.41c0-.89-1.08-1.34-1.71-.71l-.64.65z'
        }
      }
    },
    xAxis: {
      type: 'time',
      boundaryGap: [0, 0],
      axisLine: {lineStyle: {color: '#ddd'}},
      axisLabel: {
        color: '#666',
        margin: 12,
        hideOverlap: true,
        formatter: (value: number) => {
          const range = zoomRange.value;
          const spanMs = range ? range[1] - range[0] : fullSpanMs;
          return formatTimeTick(value, spanMs);
        }
      },
      splitLine: {show: false}
    },
    yAxis: {
      type: yAxisType,
      scale: true,
      splitLine: {lineStyle: {type: 'dashed', color: '#eee'}},
      name: unit ?? undefined,
      nameTextStyle: {color: '#80868b'},
      nameGap: 10,
      axisLabel: {color: '#666'}
    },
    series: [{
      name: seriesName,
      data: seriesData,
      type: 'line',
      smooth: chartPrefs.smooth,
      connectNulls: chartPrefs.connectNulls,
      showSymbol: false,
      symbol: 'circle',
      symbolSize: 6,
      sampling: samplingSafe as any,
      lineStyle: {width: 1},
      itemStyle: {color: '#2080f0'},
      areaStyle: showArea
        ? {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              {offset: 0, color: 'rgba(32, 128, 240, 0.22)'},
              {offset: 1, color: 'rgba(32, 128, 240, 0.01)'}
            ])
          }
        : null,
      markPoint: {
        label: {
          show: false
        },
        emphasis: {
          label: {
            show: true,
            formatter: (p: any) => {
              const name = p?.name ?? '';
              const value = p?.value;
              const valText = typeof value === 'number' && Number.isFinite(value)
                ? formatValueByUnit(value, unit)
                : '-';
              return name ? `${name}: ${valText}` : `${valText}`;
            }
          }
        },
        data: [
          {type: 'max', name: '最大值', itemStyle: {color: '#d03050'}},
          {type: 'min', name: '最小值', itemStyle: {color: '#18a058'}}
        ]
      }
    }]
  } as EChartsOption, true);

  chartReady.value = true;

  if (chartPrefs.yAxisScale === 'log' && yAxisType !== 'log' && !logFallbackWarned.value) {
    logFallbackWarned.value = true;
    message.warning('当前数据包含 ≤ 0 的值，无法使用对数坐标轴（已自动回退为线性）。');
  }
  if (yAxisType === 'log') {
    logFallbackWarned.value = false;
  }
};


const onResize = () => chartInstance?.resize();

const onDataZoom = (evt: any) => {
  if (zoomSyncLocked.value) return;

  const batch: any[] = (evt?.batch && Array.isArray(evt.batch) && evt.batch.length)
    ? evt.batch
    : [evt];

  // Prefer the item that provides axis values; some batches only include percent (start/end)
  const withValues = batch.find(b => b?.startValue != null && b?.endValue != null);
  const payload = withValues ?? batch[0];

  let startValue = normalizeZoomValue(payload?.startValue);
  let endValue = normalizeZoomValue(payload?.endValue);

  // Fallback: compute by percent if we can.
  if ((startValue == null || endValue == null) && fullTimeRange.value) {
    const startPct = typeof payload?.start === 'number' ? payload.start : null;
    const endPct = typeof payload?.end === 'number' ? payload.end : null;
    if (startPct != null && endPct != null) {
      const [minTs, maxTs] = fullTimeRange.value;
      const span = maxTs - minTs;
      startValue = minTs + (span * startPct) / 100;
      endValue = minTs + (span * endPct) / 100;
    }
  }

  if (startValue == null || endValue == null) return;
  zoomSyncLocked.value = true;
  try {
    timeRange.value = [startValue, endValue];
  } finally {
    zoomSyncLocked.value = false;
  }
};

async function loadAndBuild(rawKey: string) {
  rawData.value = await getData(rawKey);
  const formattedName = extractFormattedName(rawKey);
  sensorFieldName.value = formattedName;

  const {points, field, invalidTime, missingValue} = buildPoints(rawData.value, formattedName);
  invalidTimeCount.value = invalidTime;
  missingValueCount.value = missingValue;
  pointsAll.value = points;

  if (!field && rawData.value.length) {
    message.warning('未能识别该传感器字段，可能是后端返回结构变化。');
  }

  if (points.length) {
    fullTimeRange.value = [points[0][0], points[points.length - 1][0]];
    if (!timeRange.value) resetTimeRange();
  } else {
    fullTimeRange.value = null;
    timeRange.value = null;
  }
}

onMounted(async () => {
  if (!chartRef.value) return;
  chartInstance = echarts.init(chartRef.value);
  chartInstance.on('dataZoom', onDataZoom);
  window.addEventListener('resize', onResize);

  const initKey = route.params.fieldKey as string;
  await loadAndBuild(initKey);
  await renderChart();
});

watch(
    () => route.params.fieldKey,
    async (newKey, oldKey) => {
      console.log('fieldKey 更新为:', newKey)
      if (newKey && newKey !== oldKey) {
        timeRange.value = null;
        await loadAndBuild(newKey as string);
        await renderChart();
      }
    },
    {immediate: false}
)

watch(
  () => [chartPrefs.sampling, chartPrefs.smooth, chartPrefs.showArea, chartPrefs.connectNulls, chartPrefs.yAxisScale, pointsAll.value.length],
  async () => {
    await renderChart();
  }
)

watch(
  () => timeRange.value,
  (range) => {
    if (zoomSyncLocked.value) return;
    if (!range) {
      if (fullTimeRange.value) applyZoomRangeToChart(fullTimeRange.value);
      return;
    }
    // clamp
    const full = fullTimeRange.value;
    let [start, end] = range;
    if (full) {
      start = Math.max(full[0], start);
      end = Math.min(full[1], end);
      if (start > end) [start, end] = [full[0], full[1]];
    }
    applyZoomRangeToChart([start, end]);
  },
  {flush: 'sync'}
)

watch(
  () => chartPrefs.yAxisScale,
  () => {
    logFallbackWarned.value = false;
  }
)

onUnmounted(() => {
  window.removeEventListener('resize', onResize);
  chartInstance?.off('dataZoom', onDataZoom);
  chartInstance?.dispose();
});
</script>

<style scoped>
.sensor-chart-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.chart-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.chart-card :deep(.n-card__content) {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.chart-toolbar {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid #f0f2f5;
}

.toolbar-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.toolbar-row + .toolbar-row {
  margin-top: 8px;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.toolbar-group--grow {
  flex: 1;
  min-width: 260px;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
}

.toolbar-label {
  font-size: 12px;
  color: #80868b;
  white-space: nowrap;
}

.toolbar-date {
  width: 100%;
  min-width: 260px;
}

.toolbar-select {
  min-width: 160px;
}

.toolbar-select--narrow {
  min-width: 120px;
}

.chart-meta {
  margin-top: 8px;
  font-size: 12px;
  color: #80868b;
  line-height: 1.4;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
}

.meta-sep {
  margin: 0 8px;
}

.chart {
  width: 100%;
  flex: 1;
  min-height: 200px;
}

.empty-chart {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
</style>