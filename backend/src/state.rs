use std::sync::Arc;
//AppState：数据库连接池 + 验证码存储
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use sqlx::{MySql, Pool};

//单条验证码记录：存储正确答案+过期时间
#[derive(Clone, Debug)]
pub struct CaptchaEntry {
    pub answer: String,
    pub expires_at: DateTime<Utc>,
}

//验证码存储
//DashMap是并发安全的HashMap
#[derive(Clone, Debug, Default)]
pub struct CaptchaStore {
    pub map: DashMap<String, CaptchaEntry>,
}
impl CaptchaStore {
    //插入一条验证码记录
    pub fn insert(&self, id: String, entry: CaptchaEntry) {
        self.map.insert(id, entry);
    }

    //验证码校验
    //找不到/过期(删除)/不匹配->失败
    //匹配->成功(删除)
    pub fn verify_and_consume(&self, id: &str, user_input: &str) -> bool {
        let now = Utc::now();

        //没找到captcha_id
        let Some(entry) = self.map.get(id) else {
            return false;
        };

        //有id但是过期了
        if entry.expires_at < now {
            //删掉
            drop(entry);
            self.map.remove(id);
            return false;
        }

        //校验答案：忽略大小写+去空格
        let ok = entry.answer.eq_ignore_ascii_case(user_input.trim());

        drop(entry);

        if ok {
            //验证码用掉就删除
            self.map.remove(id);
        }

        ok
    }
    //定时清理过期验证码(后续给后台定时任务使用)
    pub fn cleanup_expired(&self) {
        let now = Utc::now();
        self.map.retain(|_, v| v.expires_at >= now);
    }
}

//全局状态：让所有的handler都可以通过Depot拿到他
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub captcha_store: Arc<CaptchaStore>,
    pub debug_captcha: bool,
    //JWT配置*登录注册接口需要用
    pub jwt_secret: String,
    pub jwt_expire_seconds: i64,
}
