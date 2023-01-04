use crate::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use rgb_lib::wallet::Transfer;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct TransferParams {
    asset_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransferItem {
}

impl TransferItem {
    pub fn new(transfer: Transfer) -> TransferItem {
        TransferItem {
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransferResult {
    transfers: Vec<TransferItem>,
}

#[get("/wallet/transfers")]
pub async fn get(
    params: web::Json<TransferParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            data.lock().unwrap().wallet.as_mut().unwrap().list_transfers(
                params.asset_id.clone(),
            )
        })
        .await
        .unwrap()
        {
            Ok(transfers) => HttpResponse::Ok().json(TransferResult {
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

        let params = TransferParams {
            asset_id: "".to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/transfers")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
