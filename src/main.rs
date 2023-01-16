use crate::wallet::ShiroWallet;
use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

mod healthz;
mod keys;
mod wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let shiro_wallet = Mutex::new(wallet::ShiroWallet::new());
        let data = web::Data::new(shiro_wallet);
        App::new()
            .app_data(data)
            .service(healthz::get)
            .service(keys::post)
            .service(keys::put)
            .service(wallet::address::get)
            .service(wallet::asset_balance::get)
            .service(wallet::assets::get)
            .service(wallet::blind::put)
            .service(wallet::data::get)
            .service(wallet::dir::get)
            .service(wallet::drain_to::put)
            .service(wallet::go_online::put)
            .service(wallet::issue::rgb20::put)
            .service(wallet::refresh::post)
            .service(wallet::send::post)
            .service(wallet::put)
            .service(wallet::transfers::get)
            .service(wallet::unspents::get)
            .service(wallet::utxos::put)
    })
    .bind("0.0.0.0:8080")?
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
