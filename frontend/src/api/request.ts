//Axios实例和拦截器

import axios from 'axios';
import { message } from "ant-design-vue";
import { useAuthStore } from "../stores/auth";
import { pinia } from "@/stores";

const baseURL =  "";

const request = axios.create({
    baseURL,
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
        if (!err?.response) {
            message.error("网络错误，无法连接服务器");
            return Promise.reject(err);
        }

        const status = err.response.status;
        //尝试从后端拿message
        const backendMsg: string | undefined = err.response.data?.message;

        if (status == 401) {
            const auth = useAuthStore(pinia);
            auth.logout();
            message.warning(backendMsg || "登录已失效，请重新登录");
        } else if (status === 400) {
            message.warning(backendMsg || "请求参数错误");
        } else if (status === 409) {
            message.warning(backendMsg || "数据冲突（可能用户名/邮箱已存在）");
        } else if (status >= 500) {
            message.error(backendMsg || "服务器错误");
        } else {
            // 其他状态码：兜底提示
            message.warning(backendMsg || `请求失败(${status})`);
        }

        return Promise.reject(err);
    }
);
export default request;