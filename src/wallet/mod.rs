use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Online, Wallet, WalletData};
use serde::Deserialize;
use serde::Serialize;
use std::sync::{Arc, RwLock};

pub mod address;
pub mod data;
pub mod dir;
pub mod go_online;

pub struct WalletState {
    wallet_data: Option<WalletData>,
    online: Option<Online>,
}

impl WalletState {
    pub fn new() -> Self {
        WalletState {
            wallet_data: None,
            online: None,
        }
    }

    pub async fn update(&mut self, pubkey: String, mnemonic: String) -> WalletData {
        let base_data = shiro_backend::opts::get_wallet_data();
        let wallet_data = WalletData {
            data_dir: base_data.data_dir,
            bitcoin_network: base_data.bitcoin_network,
            database_type: base_data.database_type,
            pubkey,
            mnemonic: Some(mnemonic),
        };
        if self._new_wallet(wallet_data.clone()).await.is_ok() {
            self.wallet_data = Some(wallet_data.clone());
        }
        wallet_data
    }

    pub async fn new_wallet(&self) -> Option<Wallet> {
        let wallet_data = &self.wallet_data;
        if let Some(wallet_data) = wallet_data {
            if let Ok(wallet) = self._new_wallet(wallet_data.clone()).await {
                return Some(wallet);
            }
        }
        None
    }

    async fn _new_wallet(&self, wallet_data: WalletData) -> Result<Wallet, rgb_lib::Error> {
        actix_web::rt::task::spawn_blocking(move || Wallet::new(wallet_data))
            .await
            .unwrap()
    }

    pub async fn update_online(
        &mut self,
        skip_contestency_check: bool,
        electrum_url: String,
        proxy_url: String,
    ) -> Result<Online, rgb_lib::Error> {
        let wallet = self.new_wallet().await;
        if let Some(wallet) = wallet {
            match Self::_go_online(wallet, skip_contestency_check, electrum_url, proxy_url).await {
                Ok(online) => {
                    self.online = Some(online.clone());
                    Ok(online)
                }
                Err(e) => Err(e),
            }
        } else {
            Err(rgb_lib::Error::InvalidOnline())
        }
    }

    async fn _go_online(
        mut wallet: Wallet,
        skip_contestency_check: bool,
        electrum_url: String,
        proxy_url: String,
    ) -> Result<Online, rgb_lib::Error> {
        actix_web::rt::task::spawn_blocking(move || {
            wallet.go_online(skip_contestency_check, electrum_url, proxy_url)
        })
        .await
        .unwrap()
    }

    #[allow(dead_code)]
    pub fn get_online(&self) -> Option<Online> {
        self.online.clone()
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
#[allow(clippy::await_holding_lock)]
pub async fn put(
    params: web::Json<WalletParams>,
    arc: web::Data<Arc<RwLock<WalletState>>>,
) -> impl Responder {
    if let Ok(mut wallet_state) = arc.write() {
        wallet_state
            .update(params.pubkey.clone(), params.mnemonic.clone())
            .await;
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

    use actix_web::{http, test, web, App};

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
