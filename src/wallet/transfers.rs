use crate::ShiroWallet;
use actix_web::{get, web, HttpResponse, Responder};
use rgb_lib::{
    wallet::{Outpoint, TransferKind},
    TransferStatus,
};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct TransferParams {
    asset_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Transfer {
    idx: String,
    created_at: String,
    updated_at: String,
    status: String,
    amount: String,
    kind: String,
    txid: Option<String>,
    blinded_utxo: Option<String>,
    unblinded_utxo: Option<Outpoint>,
    change_utxo: Option<Outpoint>,
    blinding_secret: Option<String>,
    expiration: Option<String>,
}

impl From<rgb_lib::wallet::Transfer> for Transfer {
    fn from(x: rgb_lib::wallet::Transfer) -> Transfer {
        Transfer {
            idx: x.idx.to_string(),
            created_at: x.created_at.to_string(),
            updated_at: x.updated_at.to_string(),
            status: match x.status {
                TransferStatus::WaitingCounterparty => "WaitingCounterparty",
                TransferStatus::WaitingConfirmations => "WaitingConfirmations",
                TransferStatus::Settled => "Settled",
                TransferStatus::Failed => "Failed",
            }
            .to_string(),
            amount: x.amount.to_string(),
            kind: match x.kind {
                TransferKind::Issuance => "issuance",
                TransferKind::Receive => "receive",
                TransferKind::Send => "send",
            }
            .to_string(),
            txid: x.txid,
            blinded_utxo: x.blinded_utxo,
            unblinded_utxo: x.unblinded_utxo,
            change_utxo: x.change_utxo,
            blinding_secret: x.blinding_secret.map(|n| n.to_string()),
            expiration: x.expiration.map(|n| n.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransferResult {
    transfers: Vec<Transfer>,
}

#[get("/wallet/transfers")]
pub async fn get(
    params: web::Json<TransferParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            data.lock()
                .unwrap()
                .wallet
                .as_mut()
                .unwrap()
                .list_transfers(params.asset_id.clone())
        })
        .await
        .unwrap()
        {
            Ok(transfers) => HttpResponse::Ok().json(
                transfers
                    .into_iter()
                    .map(Transfer::from)
                    .collect::<Vec<Transfer>>(),
            ),
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
        fund_wallet(address.new_address);
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
            let params = UtxosParams::new(true, Some(1), None, 1.0);
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
                presision: 8,
                amounts: vec![100.to_string()],
            };
            let req = test::TestRequest::put()
                .uri("/wallet/issue/rgb20")
                .set_json(params)
                .to_request();
            test::call_and_read_body_json(&app, req).await
        };
        let params = TransferParams {
            asset_id: rgb20_result.asset_id,
        };
        let req = test::TestRequest::get()
            .uri("/wallet/transfers")
            .set_json(params)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
