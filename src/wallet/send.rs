use crate::ShiroWallet;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct SendParams {
    recipient_map: HashMap<String, Vec<Recipient>>,
    donation: bool,
}

#[derive(Serialize, Deserialize)]
struct Recipient {
    blinded_utxo: String,
    amount: String,
}

impl Recipient {
    pub fn conv(&self) -> rgb_lib::wallet::Recipient {
        rgb_lib::wallet::Recipient {
            blinded_utxo: self.blinded_utxo.clone(),
            amount: str::parse::<u64>(&self.amount).unwrap(),
        }
    }
}

impl From<rgb_lib::wallet::Recipient> for Recipient
{
    fn from(x: rgb_lib::wallet::Recipient) -> Recipient {
        Recipient {
            blinded_utxo: x.blinded_utxo,
            amount: x.amount.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SendResult {
    txid: String,
}

#[post("/wallet/send")]
pub async fn post(
    params: web::Json<SendParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        if data.lock().unwrap().online.is_some() {
            match actix_web::rt::task::spawn_blocking(move || {
                let mut shiro_wallet = data.lock().unwrap();
                let online = shiro_wallet.get_online().unwrap();
                let recipient_map = params.recipient_map.iter().map(|(psbt, recipients)| (psbt.clone(), recipients.iter().map(|recipient| recipient.conv()).collect::<Vec<rgb_lib::wallet::Recipient>>())).collect::<HashMap<_, _>>();
                println!("repicient_map: {:#?}", recipient_map);
                println!("donation: {:#?}", params.donation);
                shiro_wallet.wallet.as_mut().unwrap().send(
                    online,
                    recipient_map,
                    params.donation,
                )
            })
            .await
            .unwrap()
            {
                Ok(txid) => HttpResponse::Ok().json(SendResult { txid, }),
                Err(e) => {
                    println!("{}", e);
                    HttpResponse::BadRequest().body(e.to_string())}
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
        tests::fund_wallet,
        utxos::UtxosParams,
    };
    use actix_web::{test, web, App};
    use rgb_lib::{generate_keys, wallet::{Wallet, WalletData}};

    async fn get_blinded_utxo() -> String {
        let keys = generate_keys(rgb_lib::BitcoinNetwork::Regtest);
        let base_data = shiro_backend::opts::get_wallet_data();
        let wallet_data = WalletData {
            data_dir: base_data.data_dir,
            bitcoin_network:base_data.bitcoin_network,
            database_type: base_data.database_type,
            pubkey: keys.xpub,
            mnemonic: Some(keys.mnemonic),
        };
        actix_web::rt::task::spawn_blocking(move || {
            let mut wallet = Wallet::new(wallet_data).unwrap();
            let address = wallet.get_address();
            fund_wallet(address);
            let online = wallet.go_online(true,
                    "127.0.0.1:50001".to_string(),
                    "http://proxy.rgbtools.org".to_string()).unwrap();
            wallet.create_utxos(online, true, Some(1), None).unwrap();
            let blind_data = wallet.blind(None, None, None).unwrap();
            //let blind_data = wallet.blind(Some(asset_id), Some(10), None).unwrap();
            blind_data.blinded_utxo
        }).await.unwrap()
    }


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
                .service(crate::wallet::blind::put)
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
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        let rgb20_result: Rgb20Result = {
            let params = Rgb20Params {
                ticker: "FAKEMONA".to_string(),
                name: "Fake Monacoin".to_string(),
                presision: 7,
                amounts: vec![666.to_string()],
            };
            let req = test::TestRequest::put()
                .uri("/wallet/issue/rgb20")
                .set_json(params)
                .to_request();
            test::call_and_read_body_json(&app, req).await
        };
            println!("{:#?}", rgb20_result);
        let blinded_utxo = get_blinded_utxo().await;
        let mut recipient_map = HashMap::new();
        recipient_map.insert(rgb20_result.asset_id, vec!(Recipient { blinded_utxo, amount: "10".to_string()}));
        let params = SendParams {
            recipient_map,
            donation: false,
        };
        let req = test::TestRequest::post()
            .uri("/wallet/send")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
