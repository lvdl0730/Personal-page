//创建路由，注入全局状态
mod config;
mod handlers;
mod state;

use salvo::affix_state;
use salvo::prelude::*;
use tokio::time::{Duration as TokioDuration, sleep};

use config::{database::create_mysql_pool, settings::Settings};
use state::{AppState, CaptchaStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //读取配置
    let settings = Settings::from_env()?;
    //建立数据库连接池
    let db = create_mysql_pool(&settings.database_url).await?;
    //创建验证码存储（存在进程内存)
    let captcha_store = CaptchaStore::default();

    //组装全局状态，注入到salvo的Depot里
    let state = AppState {
        db,
        captcha_store: captcha_store.clone(),
        debug_captcha: settings.debug_captcha,
    };

    //每60s清理一次过期验证码
    tokio::spawn(async move {
        loop {
            sleep(TokioDuration::from_secs(60)).await;
            captcha_store.cleanup_expired();
        }
    });

    //路由
    let router = Router::new()
        //健康检测
        .push(Router::with_path("health").get(handlers::health::health))
        //验证码
        .push(Router::with_path("api/captcha").get(handlers::captcha::get_captcha))
        //验证码校验
        .push(Router::with_path("api/captcha/verify").post(handlers::captcha::verify_captcha))
        //注入全局状态：让全部的handler都能拿到Appstate
        .hoop(affix_state::inject(state));

    //启动服务
    let addr = format!("{}:{}", settings.server_host, settings.server_port);
    println!("Server running at http://{addr}");

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(router).await;
    Ok(())
}

// mod affix_state {
//     use crate::state::AppState;
//     use salvo::prelude::*;

//     pub fn inject(state: AppState) -> impl Handler {
//         move |depot: &mut Depot, _req: &mut Request, _res: &mut Response, ctrl: &mut FlowCtrl| {
//             depot.inject(state.clone());
//             ctrl.call_next();
//         }
//     }
// }
