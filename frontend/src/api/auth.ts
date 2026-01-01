//这个部分专门放登录、注册、验证码、用户信息

import request from './request';

//验证码部分
//captcha_id：让后端生成验证码的唯一id
//image：验证码图片
export type CaptchaResp = { captcha_id: string; image: string };
export async function getCaptcha() {
    const { data } = await request.get<CaptchaResp>('/api/captcha');
    return data;
}

//登录部分
//和登录页面提交字段对齐
export type LoginReq = {
    account: string,
    password: string,
    captcha_id: string,
    captcha: string,
};
export type LoginResp = { token: string };
export async function loginApi(payload: LoginReq) {
    const { data } = await request.post<LoginResp>("/api/auth/login", payload);
    return data;
}

//注册部分
//和注册页面提交字段对齐
export type RegisterReq = {
    username: string,
    email: string,
    password: string,
    captcha_id: string,
    captcha: string,
};
export type RegisterResp = { token: string }
export async function registerApi(payload: RegisterReq) {
    const { data } = await request.post<RegisterResp>("/api/auth/register", payload);
    return data;
}

//获取当前用户信息，用来检查token是否有效
export async function meApi() {
    const { data } = await request.get("/api/auth/me");
    return data;
}