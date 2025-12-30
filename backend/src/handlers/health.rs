use salvo::prelude::*;
use serde::Serialize;

use crate::state::AppState;

//返回给前端的健康信息
#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
    db_ok: bool,
}

#[handler]
pub async fn health(depot: &Depot) -> Json<HealthResp> {
    //从全局状态拿数据库连接池
    let state = depot.obtain::<AppState>().expect("AppState未注入");

    //readiness：测试数据库是否可用
    //使用的是SELECT 1最便宜的探活方式
    let db_ok = sqlx::query("Select 1").execute(&state.db).await.is_ok();

    //liveness：存活状态只要进到这里面，就说明Web服务活着
    let status = if db_ok { "ok" } else { "degraded" };

    Json(HealthResp { status, db_ok })
}
