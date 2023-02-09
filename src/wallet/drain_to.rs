use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct DrainToParams {
    address: String,
    destroy_assets: bool,
    fee_rate: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DrainToResult {
    txid: String,
}

#[put("/wallet/drain_to")]
pub async fn put(
    params: web::Json<DrainToParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        if data.lock().unwrap().online.is_some() {
            match actix_web::rt::task::spawn_blocking(move || {
                let mut shiro_wallet = data.lock().unwrap();
                let online = shiro_wallet.get_online().unwrap();
                shiro_wallet.wallet.as_mut().unwrap().drain_to(
                    online,
                    params.address.clone(),
                    params.destroy_assets,
                    params.fee_rate,
                )
            })
            .await
            .unwrap()
            {
                Ok(txid) => HttpResponse::Ok().json(DrainToResult { txid }),
                Err(e) => HttpResponse::BadRequest().body(e.to_string()),
            }
        } else {
            HttpResponse::BadRequest().body("wallet should be online")
        }
    } else {
        HttpResponse::BadRequest().body("wallet should be created first")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::wallet::{address::AddressResult, go_online::GoOnlineParams, WalletParams};
    use actix_web::{http, test, web, App};
    use rgb_lib::generate_keys;

    #[actix_web::test]
    async fn test_put_failed() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put)
                .service(crate::wallet::go_online::put)
                .service(crate::wallet::address::get)
                .service(crate::wallet::put),
        )
        .await;

        {
            let keys = generate_keys(rgb_lib::BitcoinNetwork::Regtest);
            let wallet_params = WalletParams {
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
        }
        {
            let params = GoOnlineParams::new(true, "127.0.0.1:50001".to_string());
            let req = test::TestRequest::put()
                .uri("/wallet/go_online")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        let address: AddressResult = {
            let req = test::TestRequest::get().uri("/wallet/address").to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
            test::read_body_json(resp).await
        };
        {
            let params = DrainToParams {
                address: address.new_address,
                destroy_assets: false,
                fee_rate: 0.0,
            };
            let req = test::TestRequest::put()
                .uri("/wallet/drain_to")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }
    }

    #[actix_web::test]
    #[ignore]
    async fn test_put_success() {}
}
