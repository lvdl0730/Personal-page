//Axios实例和拦截器

import axios from 'axios';
import {message} from "ant-design-vue";
import {useAuthStore} from "../stores/auth";
import {pinia} from "../stores";

const request = axios.create({
    baseURL: import.meta.env.VUE_APP_API_URL || "",
    timeout: 10000,
});

//请求拦截器:每次发请求前都自动带上Authorization
//Header长这样
// Authorization:Bearer<token>
request.interceptors.request.use((config) => {
    const auth = useAuthStore(pinia);
    if (auth.token) {
        config.headers = config.headers ?? {};
        config.headers.Authorization = `Bearer ${auth.token}`;
    }
    return config;
});

//响应拦截器：统一处理错误，重点处理401
//报401错误说明token过期/无效，此时就退出登录，并且弹出提示
request.interceptors.response.use(
    (resp) => resp,
    (err) => {
        const status = err?.response?.status;
        if (status == 401) {
            const auth = useAuthStore(pinia);
            auth.logout();
            message.warning("登录已失效，请重新登录");
        }
        return Promise.reject(err);
    }
);
export default request;