use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Wallet, WalletData};
use serde::Deserialize;
use serde::Serialize;
use std::sync::{Arc, RwLock};

pub mod dir;

pub struct WalletState {
    wallet_data: Option<WalletData>,
}

impl WalletState {
    pub fn new() -> Self {
        WalletState { wallet_data: None }
    }

    pub fn update(&mut self, pubkey: String, mnemonic: String) -> WalletData {
        let base_data = shiro_backend::opts::get_wallet_data();
        let wallet_data = WalletData {
            data_dir: base_data.data_dir,
            bitcoin_network: base_data.bitcoin_network,
            database_type: base_data.database_type,
            pubkey,
            mnemonic: Some(mnemonic),
        };
        if Wallet::new(wallet_data.clone()).is_ok() {
            self.wallet_data = Some(wallet_data.clone());
        }
        wallet_data
    }

    pub fn exists(&self) -> bool {
        self.wallet_data.is_some()
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
    arc: web::Data<Arc<RwLock<WalletState>>>,
) -> impl Responder {
    if let Ok(mut wallet_state) = arc.write() {
        wallet_state.update(params.pubkey.clone(), params.mnemonic.clone());
        if wallet_state.exists() {
            HttpResponse::Ok().json(params)
        } else {
            HttpResponse::BadRequest().body("")
        }
    } else {
        HttpResponse::BadRequest().body("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body, body::MessageBody as _, http, rt::pin, test, web, App};

    #[actix_web::test]
    async fn test_put_failed() {
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
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
    async fn test_put_bad() {
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
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
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
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
