use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Online, Wallet, WalletData};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

pub mod address;
pub mod data;
pub mod dir;
pub mod go_online;

pub struct ShiroWallet {
    pub wallet_data: Option<WalletData>,
    pub wallet: Option<Wallet>,
    pub online: Option<Online>,
}

impl ShiroWallet {
    pub fn new() -> ShiroWallet {
        ShiroWallet {
            wallet_data: None,
            wallet: None,
            online: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WalletParams {
    mnemonic: String,
    pubkey: String,
}

#[put("/wallet")]
pub async fn put(
    params: web::Json<WalletParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    let mut shiro_wallet = data.lock().unwrap();
    match shiro_wallet.wallet {
        Some(_) => HttpResponse::BadRequest().body("wallet already created"),
        None => {
            let base_data = shiro_backend::opts::get_wallet_data();
            let wallet_data = WalletData {
                data_dir: base_data.data_dir,
                bitcoin_network: base_data.bitcoin_network,
                database_type: base_data.database_type,
                pubkey: params.pubkey.clone(),
                mnemonic: Some(params.mnemonic.clone()),
            };
            let wlt =
                actix_web::rt::task::spawn_blocking(move || Wallet::new(wallet_data).unwrap())
                    .await
                    .unwrap();
            shiro_wallet.wallet = Some(wlt);
            HttpResponse::Ok().json(params)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{http, test, web, App};

    #[actix_web::test]
    #[ignore]
    async fn test_put_failed() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put),
        )
        .await;
        let wallet_params = WalletParams {
            mnemonic: "".to_string(),
            pubkey: "".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[ignore]
    async fn test_put_bad() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put),
        )
        .await;
        let wallet_params = WalletParams {
            mnemonic: "save call film frog usual market noodle hope stomach chat word worry bad".to_string(),
            pubkey: "tpubD6NzVbkrYhZ4YT9CY6kBTU8xYWq2GQPq4NYzaJer1CRrffVLwzYt5Rs3WhjZJGKaNaiN42JfgtnyGwHXc5n5oPbAUSbxwuwDqZci5kdAZHb".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_put() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put),
        )
        .await;
        let wallet_params = WalletParams {
            mnemonic: "save call film frog usual market noodle hope stomach chat word worry".to_string(),
            pubkey: "tpubD6NzVbkrYhZ4YT9CY6kBTU8xYWq2GQPq4NYzaJer1CRrffVLwzYt5Rs3WhjZJGKaNaiN42JfgtnyGwHXc5n5oPbAUSbxwuwDqZci5kdAZHb".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
