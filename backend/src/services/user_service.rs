use anyhow::Ok;
use sqlx::FromRow;
use sqlx::{MySql, Pool};

//定义从数据库查出来的一行用户长什么样
//FromRow：可以让sqlx把SELECT选取的结果自动映射到这个结构体
//带上password_hash是因为登陆时需要校验密码
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

//创建用户(注册时使用)
//输入username、email、password_hash
//输出新用户id
pub async fn create_user(
    db: &Pool<MySql>,
    username: &str,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<i64> {
    //执行完会返回一个result，里面有last_insert_id(最后插入的id)
    let result = sqlx::query!(
        r#"INSERT INTO users (username, email, password_hash)
           VALUES (?, ?, ?)"#,
        username,
        email,
        password_hash
    )
    .execute(db)
    .await?;

    //MySQL的last_insert_id是u64
    Ok(result.last_insert_id() as i64)
}

//通过"用户名或邮箱"查询用户
pub async fn find_user_by_account(
    db: &Pool<MySql>,
    account: &str,
) -> anyhow::Result<Option<UserRow>> {
    //通过query_as把结果映射到UserRow
    //fetch_optional:查到->Some(UserRow),没查到->Ok(None)
    let user = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, username, email, password_hash
           FROM users
           WHERE username = ? OR email = ?
           LIMIT 1"#,
    )
    .bind(account)
    .bind(account)
    .fetch_optional(db)
    .await?;

    Ok(user)
}

//通过id查询用户(/me接口用)
//从token里拿到user_id,用user_id查数据库,返回用户信息
pub async fn find_user_by_id(db: &Pool<MySql>, user_id: i64) -> anyhow::Result<Option<UserRow>> {
    let user = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, username, email, password_hash
           FROM users
           WHERE id = ?
           LIMIT 1"#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    Ok(user)
}
