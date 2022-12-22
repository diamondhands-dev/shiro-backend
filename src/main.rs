use actix_web::{web, App, HttpServer};
use crate::wallet::ShiroWallet;
use std::sync::{Mutex, MutexGuard};

mod healthz;
mod keys;
mod wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
    let wallet_state = Mutex::new(wallet::ShiroWallet::new());
    App::new()
        .app_data(web::Data::new(wallet_state))
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
    use once_cell::sync::Lazy;
    use serde::Deserialize;
    use serde::Serialize;
    use std::sync::Mutex;
    use test_context::{test_context, AsyncTestContext};

    static SHIRO_WALLET: Lazy<Mutex<ShiroWallet>> = Lazy::new(|| Mutex::new(ShiroWallet::new()));
    pub(crate) struct WalletTestContext {
        pub(crate) shiro_wallet: Mutex<ShiroWallet>,
    }

    impl WalletTestContext {
        pub(crate) fn get_shiro_wallet(&mut self) -> MutexGuard<'static, ShiroWallet> {
            self.shiro_wallet.lock().unwrap()
        }
        pub(crate) fn get_wallet_state(&mut self) -> WalletState {
            self.shiro_wallet.lock().unwrap().wallet_state
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct OnlineResult {
        pub id: String,
        pub electrum_url: String,
        pub proxy_url: String,
    }

    #[async_trait::async_trait]
    impl AsyncTestContext for WalletTestContext {
        async fn setup() -> WalletTestContext {
            WalletTestContext {
                shiro_wallet: *SHIRO_WALLET,
            }
        }

        async fn teardown(self) {
            /* nothing to do */
        }
    }

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    async fn test_root(ctx: &mut WalletTestContext) {
        let shiro_wallet = ctx.get_shiro_wallet();
        let app = test::init_service(App::new().app_data(web::Data::new(shiro_wallet))).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
    }
}
