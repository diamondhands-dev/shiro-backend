use crate::wallet::WalletState;
use actix_web::{get, web, HttpResponse, Responder};
use rgb_lib::wallet::Wallet;
use serde::Deserialize;
use serde::Serialize;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct AddressResult {
    new_address: String,
}

#[allow(clippy::await_holding_lock)]
#[get("/wallet/address")]
pub async fn get(arc: web::Data<Arc<RwLock<WalletState>>>) -> impl Responder {
    if let Ok(wallet_state) = arc.write() {
        match wallet_state.new_wallet().await {
            Some(wallet) => HttpResponse::Ok().json(AddressResult {
                new_address: _new_address(wallet).await.unwrap(),
            }),
            None => HttpResponse::BadRequest().body(""),
        }
    } else {
        HttpResponse::BadRequest().body("")
    }
}

async fn _new_address(wallet: Wallet) -> Result<String, actix_web::rt::task::JoinError> {
    actix_web::rt::task::spawn_blocking(move || wallet.get_address()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body, body::MessageBody as _, http, rt::pin, test, web, App};

    #[actix_web::test]
    async fn test_get() {
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
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

        let req = test::TestRequest::get().uri("/wallet/address").to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
        let body: AddressResult = test::read_body_json(resp).await;
        assert!(body.new_address.starts_with("bcrt1"));
    }
}
