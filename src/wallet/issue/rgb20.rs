use crate::{wallet::Balance, ShiroWallet};
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Rgb20Params {
    pub ticker: String,
    pub name: String,
    pub presision: u8,
    pub amounts: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Rgb20Result {
    pub asset_id: String,
    pub ticker: String,
    pub name: String,
    pub presision: u8,
    pub balance: Balance,
}

#[put("/wallet/issue/rgb20")]
pub async fn put(
    params: web::Json<Rgb20Params>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        if data.lock().unwrap().online.is_some() {
            match actix_web::rt::task::spawn_blocking(move || {
                let mut shiro_wallet = data.lock().unwrap();
                let online = shiro_wallet.get_online().unwrap();
                shiro_wallet.wallet.as_mut().unwrap().issue_asset_rgb20(
                    online,
                    params.ticker.clone(),
                    params.name.clone(),
                    params.presision,
                    params
                        .amounts
                        .clone()
                        .into_iter()
                        .map(|str| str.parse::<u64>().unwrap_or_default())
                        .collect(),
                )
            })
            .await
            .unwrap()
            {
                Ok(asset) => HttpResponse::Ok().json(Rgb20Result {
                    asset_id: asset.asset_id,
                    ticker: asset.ticker,
                    name: asset.name,
                    presision: asset.precision,
                    balance: asset.balance.into(),
                }),
                Err(e) => {
                    println!("{:#?}", e);
                    HttpResponse::BadRequest().body(e.to_string())
                }
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

    use crate::wallet::{
        address::AddressResult, go_online::GoOnlineParams, tests::fund_wallet, utxos::UtxosParams,
    };
    use actix_web::{test, web, App};
    use rgb_lib::generate_keys;

    #[actix_web::test]
    async fn test_put() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(put)
                .service(crate::wallet::put)
                .service(crate::wallet::address::get)
                .service(crate::wallet::utxos::put)
                .service(crate::wallet::go_online::put),
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
            let params = GoOnlineParams::new(true, "127.0.0.1:50001".to_string());
            let req = test::TestRequest::put()
                .uri("/wallet/go_online")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        {
            let params = UtxosParams::new(false, Some(1), None, 1.0);
            let req = test::TestRequest::put()
                .uri("/wallet/utxos")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        {
            let params = Rgb20Params {
                ticker: "FAKEMONA".to_string(),
                name: "Fake Monacoin".to_string(),
                presision: 8,
                amounts: vec![100.to_string()],
            };
            let req = test::TestRequest::put()
                .uri("/wallet/issue/rgb20")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
    }
}
