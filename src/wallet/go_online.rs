use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct GoOnlineParams {
    skip_contestency_check: bool,
    electrum_url: String,
    proxy_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct GoOnlineResult {
    id: String,
    electrum_url: String,
    proxy_url: String,
}

#[put("/wallet/go_online")]
pub async fn put(
    params: web::Json<GoOnlineParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            (*data.lock().unwrap()).wallet.as_mut().unwrap().go_online(
                params.skip_contestency_check,
                params.electrum_url.clone(),
                params.proxy_url.clone(),
            )
        })
        .await
        .unwrap()
        {
            Ok(online) => HttpResponse::Ok().json(GoOnlineResult {
                id: online.id.to_string(),
                electrum_url: online.electrum_url.clone(),
                proxy_url: online.proxy_url.clone(),
            }),
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
            skip_contestency_check: true,
            electrum_url: "127.0.0.1:50001".to_string(),
            proxy_url: "http://proxy.rgbtools.org".to_string(),
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
            skip_contestency_check: true,
            electrum_url: "127.0.0.1:50001".to_string(),
            proxy_url: "http://proxy.rgbtools.org".to_string(),
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
