export default {
    message: {
        hello: '你好，世界',
        welcome: '欢迎使用 HWInfo Log Viewer'
    },
    common: {
        save: '保存',
        cancel: '取消',
        confirm: '确认',
        language: '语言',
        csvFile: 'CSV 文件',
        unknownError: '未知错误',
        noData: '暂无数据',
        nameValue: '{name}：{value}'
    },
    nav: {
        home: '首页',
        sensorData: '传感器数据',
        settings: '系统设置'
    },
    dashboard: {
        welcomeTitle: '欢迎使用 HWInfo Log Viewer',
        welcomeDescription: '请选择并上传 CSV 日志文件以开始分析',
        chooseCsv: '选择 CSV 文件',
        uploadAndProcess: '上传并处理',
        selectedFilePrefix: '已选择：{path}',

        dataOverviewTitle: '数据概览',
        dataLoadedTitle: '数据已加载',
        manageData: '管理数据',

        chooseCsvFirst: '请先选择 CSV 文件',
        processingCsv: '正在处理 CSV…',
        csvProcessDone: 'CSV 处理完成',
        csvProcessFailed: '处理失败：{error}',
        selectCsvFailed: '选择 CSV 文件失败：{error}',
        selectCsvSuccess: '已选择 CSV：{path}'
    },
    settings: {
        preferencesTitle: '偏好设置',
        dataManagementTitle: '数据管理',
        currentFilePath: '当前文件路径',
        noFileSelected: '未选择文件',
        chooseNewFile: '选择新文件',
        reload: '重新加载',
        clearData: '清除数据',
        clearConfirm: '确定要清除当前加载的数据吗？',

        chooseCsvFirst: '请先选择 CSV 文件',
        processingCsv: '正在处理 CSV…',
        csvProcessedOk: 'CSV 处理成功',
        csvProcessFailed: 'CSV 处理失败：{error}',
        selectCsvFailed: '选择 CSV 文件失败：{error}',
        selectCsvSuccess: '已选择 CSV：{path}',
        dataCleared: '数据已清除'
    },
    sidebar: {
        loadingData: '正在获取数据…',
        noDataHint: '暂无数据，请先上传并处理 CSV',
        dataLoaded: '数据加载成功',
        dataLoadFailed: '获取数据失败：{error}',
        parseBackendFailed: '解析后端返回失败：{error}'
    },
    chart: {
        analysisTitle: '数据分析',
        analysisTitleWithUnit: '数据分析（单位：[{unit}]）',
        min: '最小值',
        max: '最大值',
        avg: '平均值',
        median: '中位数',
        trendTitle: '趋势图',
        resetView: '重置视图',

        sampling: '降采样',
        sampling_auto: '自动',
        sampling_none: '关闭',
        sampling_average: '平均',
        sampling_max: '最大',
        sampling_min: '最小',

        yAxis: 'Y 轴',
        yAxis_linear: '线性',
        yAxis_log: '对数',

        smooth: '平滑',
        area: '面积',
        connectNulls: '连线缺失',

        warning: '预警',
        enable: '启用',
        warnMin: '下限',
        warnMax: '上限',

        metaValid: '有效值',
        metaRange: '范围：{range}',
        metaMissing: '缺失值：{count}{suffix}',
        metaMissingConnected: '（已连线）',
        metaMissingDisconnected: '（断线）',
        metaInvalidTime: '无法解析时间：{count}',

        emptyHint: '请选择左侧传感器查看数据',

        loadingSensorData: '正在加载「{name}」数据…',
        parseSensorDataFailed: '解析「{name}」数据失败：{error}',
        loadedSensorDataSuccess: '已加载「{name}」数据（{count} 条）',
        loadSensorDataFailed: '加载「{name}」数据失败：{error}',
        unrecognizedField: '未能识别该传感器字段，可能是后端返回结构变化。',
        valueSeriesName: '数值',

        seriesBelowMin: '{name}（低于下限）',
        seriesAboveMax: '{name}（高于上限）',
        lowerLimit: '下限',
        upperLimit: '上限',
        tooltipSeriesValue: '{name}：{value}',

        logAxisFallback: '当前数据包含 ≤ 0 的值，无法使用对数坐标轴（已自动回退为线性）。'
    },
    sensor: {
        current: '当前值',
        average: '平均值'
    }
}