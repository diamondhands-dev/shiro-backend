use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct UtxosParams {
    up_to: bool,
    num: Option<u8>,
    size: Option<u32>,
}

impl UtxosParams {
    pub fn new(
    up_to: bool,
    num: Option<u8>,
    size: Option<u32>) -> UtxosParams {
        UtxosParams {
            up_to,
            num,
            size,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UtxosResult {
    created_utxos: u8,
}

#[put("/wallet/utxos")]
pub async fn put(
    params: web::Json<UtxosParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        if data.lock().unwrap().online.is_some() {
            match actix_web::rt::task::spawn_blocking(move || {
                let mut shiro_wallet = data.lock().unwrap();
                let online = shiro_wallet.get_online().unwrap();
                shiro_wallet.wallet.as_mut().unwrap().create_utxos(
                    online,
                    params.up_to,
                    params.num,
                    params.size,
                )
            })
            .await
            .unwrap()
            {
                Ok(created_utxos) => HttpResponse::Ok().json(UtxosResult { created_utxos }),
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

    use crate::wallet::{address::AddressResult, go_online::GoOnlineParams, tests::fund_wallet};
    use actix_web::{test, web, App};
    use rgb_lib::generate_keys;

    #[actix_web::test]
    async fn test_put() {
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
            let params = crate::wallet::WalletParams {
                mnemonic: keys.mnemonic,
                pubkey: keys.xpub,
            };
            let req = test::TestRequest::put()
                .uri("/wallet")
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
        fund_wallet(address.new_address.clone());
        {
            let params = GoOnlineParams::new(
                true,
                "127.0.0.1:50001".to_string(),
                "http://proxy.rgbtools.org".to_string(),
            );
            let req = test::TestRequest::put()
                .uri("/wallet/go_online")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        let params = UtxosParams {
            up_to: true,
            num: Some(1),
            size: None,
        };
        let req = test::TestRequest::put()
            .uri("/wallet/utxos")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
