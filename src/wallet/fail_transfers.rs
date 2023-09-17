use crate::ShiroWallet;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Deserialize, Serialize)]
pub struct FailTransfersParams {
    blinded_utxo: Option<String>,
    txid: Option<String>,
    no_asset_only: bool,
}

#[derive(Deserialize, Serialize)]
pub struct FailTransersResult {
    transfer_changed: bool,
}

#[put("/wallet/fail_transifers")]
pub async fn put(
    params: web::Json<FailTransfersParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            let mut shiro_wallet = data.lock().unwrap();
            let online = shiro_wallet.get_online().unwrap();
            shiro_wallet.wallet.as_mut().unwrap().fail_transfers(
                online,
                params.blinded_utxo.clone(),
                params.txid.clone(),
                params.no_asset_only,
            )
        })
        .await
        .unwrap()
        {
            Ok(transfer_changed) => {
                HttpResponse::Ok().json(FailTransersResult { transfer_changed })
            }
            Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("wallet should be created first")
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    async fn test_put() {
        //TODO: implement
    }
}
