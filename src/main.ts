import { createApp } from 'vue';
import App from './App.vue';
import router from './router/index';

import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';

import { createPinia } from 'pinia';
import { i18n } from './i18n';

const app = createApp(App);

app.use(ElementPlus);
app.use(createPinia());
app.use(router);
app.use(i18n);

app.mount('#app');
