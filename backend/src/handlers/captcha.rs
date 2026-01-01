use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, Utc};
use rand::{Rng, distributions::Alphanumeric};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::{AppState, CaptchaEntry};

#[derive(Serialize)]
struct CaptchaResp {
    captcha_id: String,
    image: String,
    //多少秒后过期
    expires_in: i64,
    // //直接把答案展示出来，正式使用时删掉这部分
    // #[serde(skip_serializing_if = "Option::is_none")]
    // debug_answer: Option<String>,
}

//生成验证码：图片+id+服务端存储答案
#[handler]
pub async fn get_captcha(depot: &Depot) -> Json<CaptchaResp> {
    let state = depot.obtain::<AppState>().expect("AppState未注入");

    //生成captcha_id
    let captcha_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    //通过captcha crate生成PNG
    let mut cap = captcha::Captcha::new();
    cap.add_chars(5);
    let answer = cap.chars_as_string(); //取出答案后存储
    let png_bytes = cap.view(180, 60).as_png().expect("生成验证码图片失败");

    //服务端保存答案
    let expires_in = 120; //过期时间120s
    let entry = CaptchaEntry {
        answer: answer.clone(),
        expires_at: Utc::now() + Duration::seconds(expires_in),
    };
    state.captcha_store.insert(captcha_id.clone(), entry);

    //base64：前端可以直接显示
    let b64 = general_purpose::STANDARD.encode(png_bytes);
    let image = format!("data:image/png;base64,{}", b64);

    Json(CaptchaResp {
        captcha_id,
        image,
        expires_in,
        // debug_answer: if state.debug_captcha {
        //     Some(answer)
        // } else {
        //     None
        // },
    })
}

#[derive(Deserialize)]
struct VerifyReq {
    captcha_id: String,
    code: String,
}

#[derive(Serialize)]
struct VerifyResp {
    ok: bool,
}

//用来测试“验证码校验是否工作”
//真正的登录接口里：会把 captcha_id + code 跟用户名密码一起提交
#[handler]
pub async fn verify_captcha(req: &mut Request, depot: &Depot) -> Json<VerifyResp> {
    let state = depot.obtain::<AppState>().expect("AppState 未注入");

    //从body解析json
    let body: VerifyReq = match req.parse_json().await {
        Ok(v) => v,
        Err(_) => return Json(VerifyResp { ok: false }),
    };

    let ok = state
        .captcha_store
        .verify_and_consume(&body.captcha_id, &body.code);

    Json(VerifyResp { ok })
}
