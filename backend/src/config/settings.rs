//读取环境变量
use dotenvy::dotenv;
use std::env;

//全局配置：使用环境变量驱动
#[derive(Clone, Debug)]
pub struct Settings {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub debug_captcha: bool,
    //JWT配置
    pub jwt_secret: String,      //签名token的密钥
    pub jwt_expire_seconds: i64, //token有效期
}

impl Settings {
    //从.env+系统环境变量中读取配置
    pub fn from_env() -> anyhow::Result<Self> {
        //让本地开发可以直接用.env；线上有没有.env都没关系
        let _ = dotenv();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?;

        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("缺少环境变量DATABASE_URL"))?;

        let debug_captcha = env::var("DEBUG_CAPTCHA")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            .trim()
            == "true";

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("缺少环境变量JWT_SECRET"))?;

        let jwt_expire_seconds = env::var("JWT_EXPIRE_SECONDS")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()?;

        Ok(Self {
            server_host,
            server_port,
            database_url,
            debug_captcha,
            jwt_secret,
            jwt_expire_seconds,
        })
    }
}
