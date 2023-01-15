use crate::ShiroWallet;
use actix_web::{post, web, HttpResponse, Responder};
use rgb_lib::wallet::RefreshTransferStatus;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct RefreshParams {
    asset_id: Option<String>,
    filter: Vec<RefreshFilter>,
}

#[derive(Deserialize, Serialize)]
struct RefreshFilter {
    status: String,
    incoming: bool,
}

impl RefreshFilter {
    fn conv(&self) -> rgb_lib::wallet::RefreshFilter {
        rgb_lib::wallet::RefreshFilter {
            status: match self.status.as_str() {
                "WaitingCounterparty" => RefreshTransferStatus::WaitingCounterparty,
                "WaitingConfirmations" => RefreshTransferStatus::WaitingConfirmations,
                &_ => panic!("Unknown status"),
            },
            incoming: self.incoming,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshResult {
    result: bool,
}

#[post("/wallet/refresh")]
pub async fn post(
    params: web::Json<RefreshParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        if data.lock().unwrap().online.is_some() {
            match actix_web::rt::task::spawn_blocking(move || {
                let mut shiro_wallet = data.lock().unwrap();
                let online = shiro_wallet.get_online().unwrap();
                shiro_wallet.wallet.as_mut().unwrap().refresh(
                    online,
                    params.asset_id.clone(),
                    params
                        .filter
                        .iter()
                        .map(|x| x.conv())
                        .collect::<Vec<rgb_lib::wallet::RefreshFilter>>(),
                )
            })
            .await
            .unwrap()
            {
                Ok(result) => HttpResponse::Ok().json(RefreshResult { result }),
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
    async fn test_post() {
        let shiro_wallet = Mutex::new(ShiroWallet::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(shiro_wallet))
                .service(crate::wallet::put)
                .service(crate::wallet::address::get)
                .service(crate::wallet::utxos::put)
                .service(crate::wallet::go_online::put)
                .service(crate::wallet::issue::rgb20::put)
                .service(post),
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
        {
            let params = UtxosParams::new(false, Some(1), None);
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
        let params = RefreshParams {
            asset_id: Some(rgb20_result.asset_id),
            filter: vec![],
        };
        let req = test::TestRequest::post()
            .uri("/wallet/refresh")
            .set_json(params)
            .to_request();
        let res: RefreshResult = test::call_and_read_body_json(&app, req).await;
        assert!(res.result == false);
    }
}
