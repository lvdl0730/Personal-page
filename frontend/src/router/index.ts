import {createRouter, createWebHistory} from 'vue-router';
import {useAuthStore} from "../stores/auth.ts";
import {pinia} from "../stores";

//这三条不需要鉴权
const Login = () => import("../views/auth/Login.vue");
const Register = () => import("../views/auth/Register.vue");
const Forget = () => import("../views/auth/Forget.vue");

const BasicLayout = () => import("../layouts/BasicLayout.vue");
const Dashboard = () => import("../views/dashboard/Dashboard.vue");

//白名单：不需要登录也能访问的页面
//登录、注册、忘记密码
const WHITE_LIST = ["/login", "/register", "/forget"];

const router = createRouter({
    history: createWebHistory(),
    routes: [
        //白名单路由
        {path: '/login', component: Login, meta: {guestOnly: true}},
        {path: '/register', component: Register, meta: {guestOnly: true}},
        {path: '/forget', component: Forget, meta: {guestOnly: true}},

        //受保护路由：全部挂在BasicLayout下面
        {
            path: "/",
            component: BasicLayout,
            meta: {requiresAuth: true},
            children: [
                {path: "", redirect: "/dashboard"},
                {path: '/dashboard', component: Dashboard},
            ]
        }
    ]
})

//全局前置守卫:
//每次路由跳转都会先经过这里
//用来判断有无登录或者token是否有效
router.beforeEach(async (to) => {
    const auth = useAuthStore(pinia);

    //白名单：直接放行
    if (WHITE_LIST.includes(to.path)) {
        //如果已经登录（本地有token）就直接跳过上面三个白名单页面，直接进入到/dashboard仪表盘页面
        if (auth.token) return "/dashboard";
        return true;
    }
    //不是白名单:必须带有token
    if(!auth.token) {
        //没有token跳到登录页
        return "/login";
    }

    //如果有token，第一次进入受保护的路由时，还需要取后端验证/me
    const ok=await auth.verifyTokenOnce();
    if (!ok) {
        return "/login";
    }

    //验证通过，可以放行
    return true;
});

export default router;