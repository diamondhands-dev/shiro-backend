use crate::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use rgb_lib::wallet::Outpoint;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct UnspentsParams {
    settled_only: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Utxo {
    outpoint: Outpoint,
    btc_amount: String,
    pub colorable: bool,
}

impl From<rgb_lib::wallet::Utxo> for Utxo {
    fn from(x: rgb_lib::wallet::Utxo) -> Utxo {
        Utxo {
            outpoint: x.outpoint,
            btc_amount: x.btc_amount.to_string(),
            colorable: x.colorable,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RgbAllocation {
    asset_id: Option<String>,
    amount: String,
    settled: bool,
}

impl From<rgb_lib::wallet::RgbAllocation> for RgbAllocation {
    fn from(x: rgb_lib::wallet::RgbAllocation) -> RgbAllocation {
        RgbAllocation {
            asset_id: x.asset_id.clone(),
            amount: x.amount.to_string(),
            settled: x.settled,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct Unspent {
    utxo: Utxo,
    rgb_allocations: Vec<RgbAllocation>,
}

impl Unspent {
    fn from(origin: rgb_lib::wallet::Unspent) -> Unspent {
        Unspent {
            utxo: Utxo::from(origin.utxo),
            rgb_allocations: origin
                .rgb_allocations
                .into_iter()
                .map(RgbAllocation::from)
                .collect::<Vec<RgbAllocation>>(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UnspentsResult {
    unspents: Vec<Unspent>,
}

#[get("/wallet/unspents")]
pub async fn get(
    params: web::Json<UnspentsParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            data.lock()
                .unwrap()
                .wallet
                .as_mut()
                .unwrap()
                .list_unspents(params.settled_only)
        })
        .await
        .unwrap()
        {
            Ok(unspents) => HttpResponse::Ok().json(UnspentsResult {
                unspents: unspents
                    .into_iter()
                    .map(Unspent::from)
                    .collect::<Vec<Unspent>>(),
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

        {
            let keys = generate_keys(rgb_lib::BitcoinNetwork::Regtest);
            let wallet_params = crate::wallet::WalletParams {
                mnemonic: keys.mnemonic,
                pubkey: keys.xpub,
            };
            let req = test::TestRequest::put()
                .uri("/wallet")
                .set_json(wallet_params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
        {
            let params = UnspentsParams { settled_only: true };
            let req = test::TestRequest::get()
                .uri("/wallet/unspents")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
    }
}
