use crate::{wallet::Balance, ShiroWallet};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct AssetBalanceParams {
    asset_id: String,
}

#[get("/wallet/asset_balance")]
pub async fn get(
    params: web::Json<AssetBalanceParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            data.lock()
                .unwrap()
                .wallet
                .as_mut()
                .unwrap()
                .get_asset_balance(params.asset_id.clone())
        })
        .await
        .unwrap()
        {
            Ok(balance) => HttpResponse::Ok().json(Balance::from(balance)),
            Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("wallet should be created first")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::wallet::{
        address::AddressResult,
        go_online::GoOnlineParams,
        issue::rgb20::{Rgb20Params, Rgb20Result},
        tests::{fund_wallet, gen_fake_ticker},
        utxos::UtxosParams,
    };
    use actix_web::{test, web, App};
    use rgb_lib::generate_keys;

    #[actix_web::test]
    async fn test_get() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(crate::wallet::put)
                .service(crate::wallet::address::get)
                .service(crate::wallet::utxos::put)
                .service(crate::wallet::go_online::put)
                .service(crate::wallet::issue::rgb20::put)
                .service(get),
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
        let rgb20_result: Rgb20Result = {
            let params = Rgb20Params {
                ticker: gen_fake_ticker(),
                name: "Fake Monacoin".to_string(),
                presision: 8,
                amounts: vec![100.to_string()],
            };
            let req = test::TestRequest::put()
                .uri("/wallet/issue/rgb20")
                .set_json(params)
                .to_request();
            test::call_and_read_body_json(&app, req).await
        };
        let params = AssetBalanceParams {
            asset_id: rgb20_result.asset_id,
        };
        let req = test::TestRequest::get()
            .uri("/wallet/asset_balance")
            .set_json(params)
            .to_request();
        let asset_balance_result: Balance = test::call_and_read_body_json(&app, req).await;
        assert_eq!(asset_balance_result.settled, "100");
        assert_eq!(asset_balance_result.future, "100");
        assert_eq!(asset_balance_result.spendable, "100");
    }
}
