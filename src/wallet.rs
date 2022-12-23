use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Online, Wallet, WalletData};
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::sync::Mutex;

pub mod address;
pub mod data;
pub mod dir;

pub struct ShiroWallet {
    pub wallet_state: WalletState,
    pub online: Option<Online>,
}

impl ShiroWallet {
    pub fn new() -> ShiroWallet {
        ShiroWallet {
            wallet_state: WalletState::WalletDataE(None),
            online: None,
        }
    }
    pub fn get_wallet_state(&self) -> WalletState {
        self.wallet_state
    }
    pub fn get_online(&self) -> Option<Online> {
        self.online
    }
}

pub enum WalletState {
    WalletDataE(Option<WalletData>),
    WalletE(Wallet),
}

/*
impl fmt::Debug for WalletState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "wallet_data: ")?;
        match &self.wallet_data {
            Some(wallet_data) => {
                writeln!(f, " data_dir: {:?}", wallet_data.data_dir)?;
                //writeln!(f, " bitcoin_network: {:?}", wallet_data.bitcoin_network);
                //writeln!(f, " database_type: {:?}", wallet_data.database_type);
                writeln!(f, " pubkey: {:?}", wallet_data.pubkey)?;
                writeln!(f, " mnemonic: {:?}", wallet_data.mnemonic)?;
            },
            None => writeln!(f, " None")?
        }
        writeln!(f, "online: ")?;
        match &self.online {
            Some(online) => {
                writeln!(f, " id: {:?},", online.id)?;
                writeln!(f, " electrum_url: {:?},", online.electrum_url)?;
                writeln!(f, " proxy_url: {:?},", online.proxy_url)?;
            },
            None => writeln!(f, " None")?
        };
        Ok(())
    }
}
*/

impl WalletState {
    pub fn new() -> Self {
        Self::WalletDataE(None)
    }

    pub async fn update(&mut self, pubkey: String, mnemonic: String) -> Option<WalletData> {
        match self {
            Self::WalletDataE(wallet_data) => {
                let base_data = shiro_backend::opts::get_wallet_data();
                let wallet_data = WalletData {
                    data_dir: base_data.data_dir,
                    bitcoin_network: base_data.bitcoin_network,
                    database_type: base_data.database_type,
                    pubkey,
                    mnemonic: Some(mnemonic),
                };
                match Self::_new_wallet(wallet_data.clone()).await {
                    Ok(wallet) => {
                        let wallet_data = wallet.get_wallet_data();
                        *self = Self::WalletDataE(Some(wallet.get_wallet_data()));
                        return Some(wallet_data);
                    },
                    Err(err) => {
                        println!("{:?}", err);
                        *self = Self::WalletDataE(None);
                        return None;
                    },
                };
            },
            Self::WalletE(wallet) => {
                return Some(wallet.get_wallet_data());
            },
            _ => None,
        }
    }

    pub async fn new_address(&mut self) -> Result<String, rgb_lib::Error> {
        match self.new_wallet().await {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
        match self {
            Self::WalletDataE(_) => Err(rgb_lib::Error::Inconsistency("".to_string())),
            Self::WalletE(wallet) => match Self::_new_address(wallet).await {
                Ok(address) => Ok(address),
                Err(err) => Err(rgb_lib::Error::Inconsistency("new_address".to_string())),
            }
        }
    }

    async fn _new_address(wallet: &mut Wallet) -> Result<String, actix_web::rt::task::JoinError> {
        actix_web::rt::task::spawn_blocking(move || wallet.get_address()).await
    }

    pub async fn new_wallet(&mut self) -> Result<(), rgb_lib::Error> {
        match self {
            Self::WalletDataE(options) => {
                match options {
                    Some(wallet_data) => {
                        match Self::_new_wallet(wallet_data.clone()).await {
                            Ok(wallet) => {
                                *self = WalletState::WalletE(wallet);
                                Ok(())
                            },
                            Err(err) => Err(err),
                        }
                    },
                }
            },
            Self::WalletE(wallet) => Ok(()),
        }
    }

    async fn _new_wallet(wallet_data: WalletData) -> Result<Wallet, rgb_lib::Error> {
        actix_web::rt::task::spawn_blocking(move || Wallet::new(wallet_data))
            .await
            .unwrap()
    }

    async fn _go_online(
        wallet_data: WalletData,
        skip_contestency_check: bool,
        electrum_url: String,
        proxy_url: String,
    ) -> Result<Online, rgb_lib::Error> {
        actix_web::rt::task::spawn_blocking(move || {
            let wallet = Wallet::new(wallet_data);
            match wallet {
                Ok(mut wallet) =>wallet.go_online(skip_contestency_check, electrum_url, proxy_url),
                Err(e) => Err(e),
            }
        })
        .await
        .unwrap()
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
    arc: web::Data<Mutex<WalletState>>,
) -> impl Responder {
    if let Ok(mut wallet_state) = arc.lock() {
        match wallet_state
            .update(params.pubkey.clone(), params.mnemonic.clone())
            .await {
            Some(wallet_data) => HttpResponse::Ok().json(params),
            None => HttpResponse::BadRequest().body(""),
        }
    } else {
        HttpResponse::BadRequest().body("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{http, test, web, App};
    use crate::tests::WalletTestContext;
    use test_context::test_context;

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    #[ignore]
    async fn test_put_failed(ctx: &mut WalletTestContext) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.get_wallet_state()))
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

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    #[ignore]
    async fn test_put_bad(ctx: &mut WalletTestContext) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.get_wallet_state()))
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

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    async fn test_put(ctx: &mut WalletTestContext) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.get_wallet_state()))
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
