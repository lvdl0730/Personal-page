use std::str;

//这部分完全对齐前端的auth.ts
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::user_service;
use crate::state::AppState;
use crate::utils::auth;

//统一错误响应结构
//先统一返回{"message":"XXX"}
//给正确的HTTP码
//- 400:参数/验证码错误
//- 401:未登录/token无效/密码错误
//- 409:用户名或邮箱重复
//- 500:服务器内部错误
#[derive(Serialize)]
struct ErrorResp {
    message: &'static str,
}

//设置status code + 输出JSON错误信息
fn render_error(res: &mut Response, code: StatusCode, msg: &'static str) {
    res.status_code(code);
    res.render(Json(ErrorResp { message: msg }));
}

//请求/响应结构(和前端对齐)
//注册请求
#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub email: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha: String,
}
//token部分
#[derive(Serialize)]
pub struct TokenResp {
    pub token: String,
}
//注册请求
#[derive(Deserialize)]
pub struct LoginReq {
    pub account: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha: String,
}
//用户信息部分
#[derive(Serialize)]
pub struct MeResp {
    pub id: i64,
    pub username: String,
    pub email: String,
}

//注册:POST /api/auth/register
//流程：
// 1.parse JSON
// 2.校验验证码
// 3.参数基础校验(比如用户名、邮箱、密码格式)
// 4.hash密码
// 5.insert users插入用户
// 6.签发JWT token返回
#[handler]
pub async fn register(req: &mut Request, depot: &Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().expect("AppState未注入");
    //解析JSON body
    let body: RegisterReq = match req.parse_json().await {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::BAD_REQUEST, "请求体不是合法JSON");
            return;
        }
    };

    //验证码校验
    let captcha_ok = state
        .captcha_store
        .verify_and_consume(&body.captcha_id, &body.captcha);
    if !captcha_ok {
        render_error(res, StatusCode::BAD_REQUEST, "验证码错误或已经过期");
        return;
    }

    //基础参数校验(后续再加)
    let username = body.username.trim();
    let email = body.email.trim();
    if username.len() < 3 || username.len() > 32 {
        render_error(res, StatusCode::BAD_REQUEST, "用户名长度需要在3-32之间");
    }
    if !email.contains('@') {
        render_error(res, StatusCode::BAD_REQUEST, "邮箱格式错误");
        return;
    }
    if body.password.len() < 6 {
        render_error(res, StatusCode::BAD_REQUEST, "密码长度至少6位");
        return;
    }

    //密码哈希
    let password_hash = match auth::hash_password(&body.password) {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::BAD_REQUEST, "密码处理失败");
            return;
        }
    };

    //写入数据库
    let user_id = match user_service::create_user(&state.db, username, email, &password_hash).await
    {
        Ok(id) => id,
        Err(e) => {
            //唯一索引冲突(用户名/邮箱重复)在MySQL常见错误码是1062
            //sqlx会把它包在Database.error里
            if let Some(db_err) = e.downcast_ref::<sqlx::Error>() {
                if let sqlx::Error::Database(dbe) = db_err {
                    if dbe.code().as_deref() == Some("1062") {
                        render_error(res, StatusCode::CONFLICT, "用户名或邮箱已存在");
                        return;
                    }
                }
            }
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "数据写入错误");
            return;
        }
    };

    //签发JWT token
    let token = match auth::issue_jwt(&state.jwt_secret, state.jwt_expire_seconds, user_id) {
        Ok(t) => t,
        Err(_) => {
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "token生成失败");
            return;
        }
    };

    //返回给前端:{token}
    res.render(Json(TokenResp { token }));
}

//登录:POST /api/auth/login
//流程：
// 1.parse Json
// 2.校验验证码
// 3.通过username或email查用户
// 4.verify校验密码
// 5.签发token返回
#[handler]
pub async fn login(req: &mut Request, depot: &Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().expect("AppState未注入");

    //解析JSON body
    let body: LoginReq = match req.parse_json().await {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::BAD_REQUEST, "请求体不是合法JSON");
            return;
        }
    };

    //验证码校验
    let captcha_ok = state
        .captcha_store
        .verify_and_consume(&body.captcha_id, &body.captcha);
    if !captcha_ok {
        render_error(res, StatusCode::BAD_REQUEST, "验证码错误或已经过期");
        return;
    }

    //基础校验
    let account = body.account.trim();
    if account.is_empty() || body.password.is_empty() {
        render_error(res, StatusCode::BAD_REQUEST, "账号或密码不能为空");
        return;
    }

    //通过account查询用户
    let user = match user_service::find_user_by_account(&state.db, account).await {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "数据库查询失败");
            return;
        }
    };
    //账号不存在或密码错误统一返回账号或密码错误
    let Some(user) = user else {
        render_error(res, StatusCode::BAD_REQUEST, "账号或密码错误");
        return;
    };

    //校验密码
    let ok = match auth::verify_password(&body.password, &user.password_hash) {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "密码校验失败");
            return;
        }
    };
    if !ok {
        render_error(res, StatusCode::BAD_REQUEST, "账号或密码错误");
        return;
    };

    //签发token
    let token = match auth::issue_jwt(&state.jwt_secret, state.jwt_expire_seconds, user.id) {
        Ok(t) => t,
        Err(_) => {
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "token生成失败");
            return;
        }
    };

    res.render(Json(TokenResp { token }));
}

//Me: GET /api/auth/me
//前端用途：1.校验token是否有效  2.获取当前用户信息
//约定：
//请求头带Authorization: Bearer <token>
//验证签名后从token里拿user_id,再去数据库查用户信息返回
#[handler]
pub async fn me(req: &mut Request, depot: &Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().expect("AppState未注入");

    //解析Authorization header
    //大概长这样：Authorization: Bearer xxxxx.yyyyy.zzzzz
    let token = match parse_bearer_token(req) {
        Some(t) => t,
        None => {
            render_error(res, StatusCode::UNAUTHORIZED, "缺少token");
            return;
        }
    };

    //验证token(验证签名+检查exp)
    let claims = match auth::verify_jwt(&state.jwt_secret, &token) {
        Ok(c) => c,
        Err(_) => {
            render_error(res, StatusCode::UNAUTHORIZED, "token无效或已经过期");
            return;
        }
    };

    //使用user_id查数据库
    let user = match user_service::find_user_by_id(&state.db, claims.sub).await {
        Ok(v) => v,
        Err(_) => {
            render_error(res, StatusCode::INTERNAL_SERVER_ERROR, "数据库查询失败");
            return;
        }
    };

    let Some(user) = user else {
        render_error(res, StatusCode::UNAUTHORIZED, "用户不存在");
        return;
    };

    //返回给前端(不要返回password_hash)
    res.render(Json(MeResp {
        id: user.id,
        username: user.username,
        email: user.email,
    }));
}

//从Authorization header里解析Bearer token
fn parse_bearer_token(req: &Request) -> Option<String> {
    let raw = req.headers().get("authorization")?.to_str().ok()?;
    let raw = raw.trim();

    //允许大小写Bearer / bearer
    if raw.len() < 7 {
        return None;
    }
    if !raw[..6].eq_ignore_ascii_case("bearer") {
        return None;
    }

    //bearer后面应该是 空格+token
    let token = raw[6..].trim();
    if token.is_empty() {
        return None;
    }

    Some(token.to_string())
}
