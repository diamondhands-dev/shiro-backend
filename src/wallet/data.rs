use crate::wallet::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use rgb_lib::wallet::DatabaseType;
use rgb_lib::BitcoinNetwork;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct WalletDataResponse {
    /// Directory where the wallet directory is to be created
    pub data_dir: String,
    /// Bitcoin network for the wallet
    pub bitcoin_network: String,
    /// Database type for the wallet
    pub database_type: String,
    /// Wallet xpub
    pub pubkey: String,
    /// Wallet mnemonic phrase
    pub mnemonic: String,
}

#[get("/wallet/data")]
pub async fn get(data: web::Data<Mutex<ShiroWallet>>) -> impl Responder {
    let shiro_wallet = data.lock().unwrap();
    match &shiro_wallet.wallet {
        Some(wallet) => {
            let wdata = wallet.get_wallet_data();
            HttpResponse::Ok().json(WalletDataResponse {
                data_dir: wdata.data_dir.clone(),
                bitcoin_network: match wdata.bitcoin_network {
                    BitcoinNetwork::Mainnet => "mainnet",
                    BitcoinNetwork::Testnet => "testnet",
                    BitcoinNetwork::Regtest => "regtest",
                    BitcoinNetwork::Signet => "signet",
                }
                .to_string(),
                database_type: match wdata.database_type {
                    DatabaseType::Sqlite => "sqlite",
                }
                .to_string(),
                pubkey: wdata.pubkey.clone(),
                mnemonic: if let Some(mnemonic) = &wdata.mnemonic {
                    mnemonic.clone()
                } else {
                    "".to_string()
                },
            })
        }
        None => HttpResponse::BadRequest().body("wallet doesn't be initialized"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};

    use crate::wallet::ShiroWallet;

    #[actix_web::test]
    async fn test_get_failed() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(get)
                .service(crate::wallet::put),
        )
        .await;
        let wallet_params = crate::wallet::WalletParams {
            mnemonic: "".to_string(),
            pubkey: "".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert_eq!(wallet_resp.status(), http::StatusCode::BAD_REQUEST);

        let req = test::TestRequest::get().uri("/wallet/data").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_get() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(get)
                .service(crate::wallet::put),
        )
        .await;
        let wallet_params = crate::wallet::WalletParams {
            mnemonic: "save call film frog usual market noodle hope stomach chat word worry".to_string(),
            pubkey: "xpub661MyMwAqRbcGexM5um6FYobDPjNH1tmWjxhDkbhfHfxvNpdsmhnvzCDGfemmmNLagBTSSno9nxvaknvDDvqux8sQqrfGPGzFc2JKnf4KL9".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert!(wallet_resp.status().is_success());

        let req = test::TestRequest::get().uri("/wallet/data").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let body: WalletDataResponse = test::read_body_json(resp).await;
        assert_eq!(body.data_dir, "/tmp/shiro-wallet");
        assert_eq!(body.bitcoin_network, "regtest");
        assert_eq!(body.database_type, "sqlite");
        assert_eq!(body.pubkey, "xpub661MyMwAqRbcGexM5um6FYobDPjNH1tmWjxhDkbhfHfxvNpdsmhnvzCDGfemmmNLagBTSSno9nxvaknvDDvqux8sQqrfGPGzFc2JKnf4KL9");
        assert_eq!(
            body.mnemonic,
            "save call film frog usual market noodle hope stomach chat word worry"
        );
    }
}
