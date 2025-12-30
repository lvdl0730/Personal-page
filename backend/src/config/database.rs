//创建MySQL连接池
use sqlx::{MySql, Pool};

//创建MySQL连接池（给全站复用）
//连接池的作用:
//每次请求都新建连接非常慢，连接池提前准备一些连接，谁需要请求就拿去用，用完了还回来
pub async fn create_mysql_pool(database_url: &str) -> anyhow::Result<Pool<MySql>> {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(10) //先写10个
        .connect(database_url)
        .await?;

    //测试：确保能连的上
    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}
