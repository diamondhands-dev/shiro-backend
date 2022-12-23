use actix_web::{get, web, HttpResponse, Responder};
use crate::ShiroWallet;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct AddressResult {
    new_address: String,
}

#[allow(clippy::await_holding_lock)]
#[get("/wallet/address")]
pub async fn get(mtx: web::Data<Mutex<ShiroWallet>>) -> impl Responder {
    if let Ok(shiro_wallet) = mtx.lock() {
        let result = shiro_wallet.get_wallet_state().new_address().await;
        match result {
            Ok(address) => HttpResponse::Ok().json(AddressResult {
                new_address: address,
            }),
            Err(err) => HttpResponse::BadRequest().body("a"),
        }
    } else {
        HttpResponse::BadRequest().body("a")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{test, web, App};
    use crate::tests::WalletTestContext;
    use test_context::test_context;

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    async fn test_get(ctx: &mut WalletTestContext) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.get_wallet_state()))
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