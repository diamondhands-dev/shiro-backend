use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlindParams {
    asset_id: Option<String>,
    amount: Option<String>,
    duration_seconds: Option<u32>,
}

impl BlindParams {
    pub fn new(
    asset_id: Option<String>,
    amount: Option<String>,
    duration_seconds: Option<u32>) -> BlindParams {
BlindParams {
    asset_id,
    amount,
    duration_seconds,
        }
    }
}

pub struct BlindParamsForLib {
    asset_id: Option<String>,
    amount: Option<u64>,
    duration_seconds: Option<u32>,
}

impl From<BlindParams> for BlindParamsForLib {
    fn from(x: BlindParams) -> BlindParamsForLib {
        BlindParamsForLib {
            asset_id: x.asset_id,
            amount: x.amount.map(|str| str.parse::<u64>().unwrap()),
            duration_seconds: x.duration_seconds,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlindData {
    invoice: String,
    blinded_utxo: String,
    blinding_secret: String,
    expiration_timestamp: Option<String>,
}

impl BlindData {
    pub fn get_blinded_utxo(&self) -> String {
        self.blinded_utxo.clone()
    }
}

impl From<rgb_lib::wallet::BlindData> for BlindData {
    fn from(x: rgb_lib::wallet::BlindData) -> BlindData {
        BlindData {
            invoice: x.invoice,
            blinded_utxo: x.blinded_utxo,
            blinding_secret: x.blinding_secret.to_string(),
            expiration_timestamp: x.expiration_timestamp.map(|x| x.to_string()),
        }
    }
}

#[put("/wallet/blind")]
pub async fn put(
    params: web::Json<BlindParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            let params = BlindParamsForLib::from(params.clone());
            data.lock().unwrap().wallet.as_mut().unwrap().blind(
                params.asset_id.clone(),
                params.amount,
                params.duration_seconds,
            )
        })
        .await
        .unwrap()
        {
            Ok(blind_data) => HttpResponse::Ok().json(BlindData::from(blind_data)),
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
        tests::fund_wallet,
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
                .service(put),
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
        fund_wallet(address.new_address);
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
        {
            let params = UtxosParams::new(true, Some(1), None);
            let req = test::TestRequest::put()
                .uri("/wallet/utxos")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:#?}", resp);
            assert!(resp.status().is_success());
        }
        let rgb20_result: Rgb20Result = {
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
            test::call_and_read_body_json(&app, req).await
        };
        println!("{:#?}", rgb20_result);
        let params = BlindParams {
            asset_id: Some(rgb20_result.asset_id),
            amount: Some("10".to_string()),
            duration_seconds: Some(10),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/blind")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
