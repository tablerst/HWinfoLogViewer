export default {
    message: {
        hello: 'Hello World',
        welcome: 'Welcome to HWInfo Log Viewer'
    },
    common: {
        save: 'Save',
        cancel: 'Cancel',
        confirm: 'Confirm',
        language: 'Language',
        csvFile: 'CSV file',
        unknownError: 'Unknown error',
        noData: 'No data',
        nameValue: '{name}: {value}'
    },
    nav: {
        home: 'Home',
        sensorData: 'Sensors',
        settings: 'Settings'
    },
    dashboard: {
        welcomeTitle: 'Welcome to HWInfo Log Viewer',
        welcomeDescription: 'Select and load a CSV log file to start analyzing',
        chooseCsv: 'Choose CSV',
        uploadAndProcess: 'Load & Process',
        selectedFilePrefix: 'Selected: {path}',

        dataOverviewTitle: 'Overview',
        dataLoadedTitle: 'Data loaded',
        manageData: 'Manage data',

        chooseCsvFirst: 'Please choose a CSV file first',
        processingCsv: 'Processing CSV…',
        csvProcessDone: 'CSV processed',
        csvProcessFailed: 'Process failed: {error}',
        selectCsvFailed: 'Failed to choose CSV: {error}',
        selectCsvSuccess: 'CSV selected: {path}'
    },
    settings: {
        preferencesTitle: 'Preferences',
        dataManagementTitle: 'Data management',
        currentFilePath: 'Current file path',
        noFileSelected: 'No file selected',
        chooseNewFile: 'Choose new file',
        reload: 'Reload',
        clearData: 'Clear data',
        clearConfirm: 'Are you sure you want to clear the currently loaded data?',

        chooseCsvFirst: 'Please choose a CSV file first',
        processingCsv: 'Processing CSV…',
        csvProcessedOk: 'CSV processed',
        csvProcessFailed: 'CSV processing failed: {error}',
        selectCsvFailed: 'Failed to choose CSV: {error}',
        selectCsvSuccess: 'CSV selected: {path}',
        dataCleared: 'Data cleared'
    },
    sidebar: {
        loadingData: 'Loading data…',
        noDataHint: 'No data. Please load and process a CSV first.',
        dataLoaded: 'Data loaded',
        dataLoadFailed: 'Failed to load data: {error}',
        parseBackendFailed: 'Failed to parse backend response: {error}'
    },
    chart: {
        analysisTitle: 'Statistics',
        analysisTitleWithUnit: 'Statistics (Unit: [{unit}])',
        min: 'Min',
        max: 'Max',
        avg: 'Average',
        median: 'Median',
        trendTitle: 'Trend',
        resetView: 'Reset view',

        sampling: 'Sampling',
        sampling_auto: 'Auto',
        sampling_none: 'Off',
        sampling_average: 'Average',
        sampling_max: 'Max',
        sampling_min: 'Min',

        yAxis: 'Y axis',
        yAxis_linear: 'Linear',
        yAxis_log: 'Log',

        smooth: 'Smooth',
        area: 'Area',
        connectNulls: 'Connect nulls',

        warning: 'Alert',
        enable: 'Enable',
        warnMin: 'Min',
        warnMax: 'Max',

        metaValid: 'Valid',
        metaRange: 'Range: {range}',
        metaMissing: 'Missing: {count}{suffix}',
        metaMissingConnected: ' (connected)',
        metaMissingDisconnected: ' (disconnected)',
        metaInvalidTime: 'Unparseable time: {count}',

        emptyHint: 'Select a sensor on the left to view data',

        loadingSensorData: 'Loading "{name}"…',
        parseSensorDataFailed: 'Failed to parse "{name}": {error}',
        loadedSensorDataSuccess: 'Loaded "{name}" ({count} rows)',
        loadSensorDataFailed: 'Failed to load "{name}": {error}',
        unrecognizedField: 'Failed to recognize the sensor field. The backend response format may have changed.',
        valueSeriesName: 'Value',

        seriesBelowMin: '{name} (below min)',
        seriesAboveMax: '{name} (above max)',
        lowerLimit: 'Min',
        upperLimit: 'Max',
        tooltipSeriesValue: '{name}: {value}',

        logAxisFallback: 'The data contains values ≤ 0, so log scale cannot be used (fallback to linear).'
    },
    sensor: {
        current: 'Current',
        average: 'Average'
    }
}