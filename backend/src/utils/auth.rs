use anyhow::Ok;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

//密码哈希部分
pub fn hash_password(plain: &str) -> anyhow::Result<String> {
    //盐值salt:每个密码都要配一个随机盐
    //这样同样的密码也会哈希出不同的结果
    let salt = SaltString::generate(&mut OsRng);

    //使用Argon2默认参数
    let argon2 = Argon2::default();

    //输出是一个包含算法信息的字符串
    let password_hash = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash_password failed:{e}"))?
        .to_string();

    Ok(password_hash)
}

//密码校验部分
//从数据库读取password_hash
//对用户输入的plain password做校验
pub fn verify_password(plain: &str, password_hash: &str) -> anyhow::Result<bool> {
    //解析数据库里的hash字符串
    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| anyhow::anyhow!("PasswordHash parse failed:{e}"))?;

    //verify:返回Ok表示匹配
    let ok = Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok();

    Ok(ok)
}

//JWT部分（登录token)
//JWT是什么：
//JWT是一串字符串(token)，服务器使用secret签名，客户端每次发请求时带上它，后端就可以验证身份了
//issue(签发)：登录成功->给token
//verify(校验)：请求带token->验证签名->得到user_id
//注意：JWT不能用来存放密码之类的敏感信息，因为payload时可读的，所以只存放用户id和过期时间
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    //subject:用来存放用户id
    pub sub: i64,
    //过期时间:Unix时间戳
    pub exp: usize,
}

//签发token
pub fn issue_jwt(secret: &str, expire_seconds: i64, user_id: i64) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp(); //当前秒级时间戳
    let exp = now + expire_seconds;

    let claims = Claims {
        sub: user_id,
        exp: exp as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("jwt encode failed:{e}"))?;

    Ok(token)
}

//校验token部分：成功则返回claim
//失败原因一般有：token被篡改(验证签名失败)、token过期(exp超时)、secret不一致
pub fn verify_jwt(secret: &str, token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!("jwt decode falied:{e}"))?;

    Ok(data.claims)
}
