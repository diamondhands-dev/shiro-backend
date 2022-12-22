use crate::wallet::WalletState;
use actix_web::{post, web, HttpResponse, Responder};
use rgb_lib::wallet::{Online, Wallet};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct UtxosParams {
    up_to: bool,
    num: Option<u8>,
    size: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct UtxosResult {
    num_utxos_created: u8,
}

#[allow(clippy::await_holding_lock)]
#[post("/wallet/utxos")]
pub async fn post(
    params: web::Json<UtxosParams>,
    mtx: web::Data<Mutex<WalletState>>) -> impl Responder {
    if let Ok(mut wallet_state) = mtx.lock() {
        match wallet_state.new_wallet().await {
            Ok(wallet) => HttpResponse::Ok().json(UtxosResult {
                num_utxos_created: _utxos(wallet, online, params.into_inner()).await.unwrap(),
            }),
            Err(err) => {
                println!("{:?}", err);
                HttpResponse::BadRequest().body("aa")
            },
        }
    } else {
         HttpResponse::BadRequest().json(UtxosResult {
             num_utxos_created: 0,
         })
    }
}

async fn _utxos(mut wallet: Wallet, online: Online, params: UtxosParams) -> Result<u8, rgb_lib::Error> {
    actix_web::rt::task::spawn_blocking(move || {
        wallet.create_utxos(online, params.up_to, params.num, params.size)
    }).await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body::to_bytes, http, test, web, App};
    use actix_web::web::Bytes;
    use crate::tests::WalletTestContext;
    use test_context::test_context;

    trait BodyTest {
        fn as_str(&self) -> &str;
    }
    
    impl BodyTest for Bytes {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }

    #[test_context(WalletTestContext)]
    #[actix_web::test]
    async fn test_get(ctx: &mut WalletTestContext) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.get_wallet_state()))
                .service(post)
                .service(crate::wallet::put)
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
        assert!(wallet_resp.status().is_success());

        let req = test::TestRequest::post().uri("/wallet/utxos").set_json(UtxosParams {
            up_to: false,
            num: Some(3),
            size: Some(10),
        }).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        //let body: UtxosResult = test::read_body_json(resp).await;
        //assert_eq!(body.num_utxos_created, 0);
    }
}
