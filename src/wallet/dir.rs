use crate::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct WalletDir {
    wallet_dir: String,
}

#[get("/wallet/dir")]
pub async fn get(data: web::Data<Mutex<ShiroWallet>>) -> impl Responder {
    let shiro_wallet = data.lock().unwrap();
    match &shiro_wallet.wallet {
        Some(wallet) => HttpResponse::Ok().json(WalletDir {
            wallet_dir: wallet.get_wallet_data().data_dir,
        }),
        None => HttpResponse::BadRequest().body("wallet data has not been provided"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};

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

        let req = test::TestRequest::get().uri("/wallet/dir").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(wallet_resp.status(), http::StatusCode::BAD_REQUEST);
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
            pubkey: "tpubD6NzVbkrYhZ4YT9CY6kBTU8xYWq2GQPq4NYzaJer1CRrffVLwzYt5Rs3WhjZJGKaNaiN42JfgtnyGwHXc5n5oPbAUSbxwuwDqZci5kdAZHb".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert!(wallet_resp.status().is_success());

        let req = test::TestRequest::get().uri("/wallet/dir").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(wallet_resp.status().is_success());
    }
}
