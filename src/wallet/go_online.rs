use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct GoOnlineParams {
    skip_consistency_check: bool,
    electrum_url: String,
}

impl GoOnlineParams {
    #[allow(dead_code)]
    pub fn new(skip_consistency_check: bool, electrum_url: String) -> GoOnlineParams {
        GoOnlineParams {
            skip_consistency_check,
            electrum_url,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoOnlineResult {}

#[put("/wallet/go_online")]
pub async fn put(
    params: web::Json<GoOnlineParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            let mut shiro_wallet = data.lock().unwrap();
            let result = shiro_wallet
                .wallet
                .as_mut()
                .unwrap()
                .go_online(params.skip_consistency_check, params.electrum_url.clone());
            shiro_wallet.online = Some(result.unwrap())
        })
        .await
        {
            Ok(_) => HttpResponse::Ok().json(GoOnlineResult {}),
            Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("wallet should be created first")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{test, web, App};
    use rgb_lib::generate_keys;

    #[actix_web::test]
    async fn test_put() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put)
                .service(crate::wallet::put),
        )
        .await;

        let wallet_params = crate::wallet::WalletParams {
            mnemonic: "save call film frog usual market noodle hope stomach chat word worry".to_string(),
            pubkey: "tpubD6NzVbkrYhZ4YT9CY6kBTU8xYWq2GQPq4NYzaJer1CRrffVLwzYt5Rs3WhjZJGKaNaiN42JfgtnyGwHXc5n5oPbAUSbxwuwDqZci5kdAZHb".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert!(wallet_resp.status().is_success());

        let params = GoOnlineParams {
            skip_consistency_check: true,
            electrum_url: "127.0.0.1:50001".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/go_online")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_put_again() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put)
                .service(crate::wallet::put),
        )
        .await;

        let keys = generate_keys(rgb_lib::BitcoinNetwork::Regtest);
        let wallet_params = crate::wallet::WalletParams {
            mnemonic: keys.mnemonic,
            pubkey: keys.xpub,
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert!(wallet_resp.status().is_success());

        let params = GoOnlineParams {
            skip_consistency_check: true,
            electrum_url: "127.0.0.1:50001".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/go_online")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
