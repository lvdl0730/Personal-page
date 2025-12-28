//Pinia认证仓库

import {defineStore} from 'pinia';
import {meApi} from "../api/auth";

//“记住我”部分一般对应：
//- 记住：localStorage（关闭浏览器也还在）
//- 不记住：sessionStorage（关闭浏览器就没了）
const LS_KEY = "auth_token_ls";
const SS_KEY = "auth_token_ss";

//从两个地方尝试读取token
//优先从localStorage（因为“记住我”存在这里)
//再尝试从sessionStorage（不记住的存在这里)
function loadToken(): string {
    return localStorage.getItem(LS_KEY) || sessionStorage.getItem(SS_KEY) || "";
}

export const useAuthStore = defineStore("auth", {
    state: () => ({
        //token:JWT字符串
        //刷新页面时state状态会重建，所以初始化时从storage里读一次
        token: loadToken(),

        //user：当前登录用户信息（来自 /api/auth/me）
        //一开始不知道是谁，所以是 null
        user: null as null | Record<string, any>,

        //isTokenVerified:“我已经和后端确认过这个token时有效的”
        //防止每次路由的跳转都需要反复请求/me
        isTokenVerified: false,

        //这里不知道在干嘛
        //防抖/去重：多次并发触发 verifyTokenOnce 时，复用同一个请求
        verifyingPromise: null as null | Promise<boolean>,
    }),
    getters: {
        //只要本地有token就认为可能登录了
        //但是token不一定有效，还需要verifyTokenOnce
        isLoggedIn(state) {
            return !!state.token;
        },
    },
    actions: {
        //登录成功后调用setToken，接收后端签发的JWT和“是否记住我"
        //之后
        // 1.写state.token
        // 2.根据remember选择，存入localStorage或者sessionStorage
        // 3.清空user，并且让isTokenVerified=false，因为这是一个新token，还没有被确认过
        setToken(token: string, remember: boolean) {
            this.token = token;

            //新token还没有后端验证，先重置
            this.isTokenVerified = false;
            this.user = null;

            //清理两个storage，防止同时存在旧的token
            localStorage.removeItem(LS_KEY);
            sessionStorage.removeItem(SS_KEY);

            //根据remember选择决定存在哪里
            if (remember) {
                localStorage.setItem(LS_KEY, token);
            } else {
                sessionStorage.setItem(LS_KEY, token);
            }
        },

        //logout退出登录
        //需要把state和storage清空
        logout() {
            this.token = "";
            this.user = null;
            this.isTokenVerified = false;
            this.verifyingPromise = null;

            localStorage.removeItem(LS_KEY);
            sessionStorage.removeItem(SS_KEY);
        },

        //核心函数
        //路由守卫：进入受到保护前的页面之前需要先验证token是否真的有效
        //页面初始化：刷新后恢复登录状态
        async verifyTokenOnce(): Promise<boolean> {
            //如果没有token就返回false
            if (!this.token) return false
            //如果之前验证过就直接返回true，不需要再次请求
            if (this.isTokenVerified) return true;

            //正在验证中(有并发)：就复用同一个promise
            if (this.verifyingPromise) return this.verifyingPromise;

            this.verifyingPromise = (async () => {
                try {
                    const me = await meApi();//需要后端实现/api/auth/me
                    this.user = me;
                    this.isTokenVerified = true;
                    return true;
                } catch (e) {
                    //一旦/me失败（401或者过期),就把token认为是无效的，直接登出
                    this.logout();
                    return false;
                } finally {
                    //请求结束后清掉promise，下次需要请求再发起
                    this.verifyingPromise = null;
                }
            })();
            return this.verifyingPromise;
        }
    }
})

//最后达到的效果
//登录成功:auth.setToken(token,remember)
//需要认证:await auth.verifyTokenOnce()
//退出登录:auth.logout()