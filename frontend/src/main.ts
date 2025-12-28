import { createApp } from 'vue'
import App from './App.vue'
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/reset.css';
import router from './router';
import {pinia} from "./stores";

createApp(App)
    .use(pinia)
    .use(router)
    .use(Antd)
    .mount('#app')
