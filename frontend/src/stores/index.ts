//把pinia实例单独导出
import {createPinia} from "pinia";

//这个pinia实例会被main.ts挂到app上
//app.use(pinia)
//同时给他导出去，让axios、router这种组件外的文件也能使用
//useAuthStore(pinia)
export const pinia = createPinia();