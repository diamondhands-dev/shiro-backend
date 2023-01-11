use crate::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct AddressResult {
    pub(crate) new_address: String,
}

#[allow(clippy::await_holding_lock)]
#[get("/wallet/address")]
pub async fn get(data: web::Data<Mutex<ShiroWallet>>) -> impl Responder {
    let shiro_wallet = data.lock().unwrap();
    match &shiro_wallet.wallet {
        Some(wallet) => {
            let address = wallet.get_address();
            HttpResponse::Ok().json(AddressResult {
                new_address: address,
            })
        }
        None => HttpResponse::BadRequest().body("wallet should be created first"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{test, web, App};

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

        let req = test::TestRequest::get().uri("/wallet/address").to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
        let body: AddressResult = test::read_body_json(resp).await;
        assert!(body.new_address.starts_with("bcrt1"));
    }
}
