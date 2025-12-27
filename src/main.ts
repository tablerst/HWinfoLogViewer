import {createApp} from 'vue';
import App from './App.vue';
import router from './router/router';
import {createPinia} from 'pinia';
import {createI18n} from "vue-i18n";

import { loadPersistedLocale } from './utils/locale';

import zhCN from './locales/zh-CN.ts';
import enUS from './locales/en-US.ts';

const i18n = createI18n({
    legacy: false,
    globalInjection: true,
    locale: loadPersistedLocale('zh-CN'),
    fallbackLocale: 'en-US',
    messages: {
        'zh-CN': zhCN,
        'en-US': enUS
    }
})

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(i18n);

app.mount('#app');
