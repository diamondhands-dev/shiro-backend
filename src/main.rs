use actix_web::{web, App, HttpServer};
use crate::wallet::{ShiroWallet, WalletState};
use std::sync::{Mutex, MutexGuard, Arc};

mod healthz;
mod keys;
mod wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
    let shiro_wallet = Arc::new(Mutex::new(wallet::ShiroWallet::new()));
    App::new()
        .app_data(web::Data::new(shiro_wallet))
        .service(healthz::get)
        .service(keys::post)
        .service(keys::put)
        .service(wallet::put)
        .service(wallet::data::get)
        .service(wallet::dir::get)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{test, web, App};
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Serialize, Deserialize)]
    pub struct OnlineResult {
        pub id: String,
        pub electrum_url: String,
        pub proxy_url: String,
    }

    #[actix_web::test]
    async fn test_root() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(App::new().app_data(web::Data::new(shiro_wallet))).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
    }
}
