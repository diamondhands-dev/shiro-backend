use crate::{wallet::Balance, ShiroWallet};
use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::AssetSchema;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Deserialize, Serialize)]
pub struct AssetsParams {
    filter_asset_types: Vec<AssetSchema>,
}

#[derive(Deserialize, Serialize)]
pub struct Media {
    file_path: String,
    mime: String,
}

impl From<rgb_lib::wallet::Media> for Media {
    fn from(x: rgb_lib::wallet::Media) -> Media {
        Media {
            file_path: x.file_path,
            mime: x.mime,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AssetRgb20 {
    asset_id: String,
    ticker: String,
    name: String,
    precision: u8,
    balance: Balance,
}

impl From<rgb_lib::wallet::AssetNIA> for AssetRgb20 {
    fn from(origin: rgb_lib::wallet::AssetNIA) -> AssetRgb20 {
        AssetRgb20 {
            asset_id: origin.asset_id,
            ticker: origin.ticker,
            name: origin.name,
            precision: origin.precision,
            balance: Balance::from(origin.balance),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AssetRgb25 {
    asset_id: String,
    name: String,
    precision: u8,
    description: Option<String>,
    balance: Balance,
    data_paths: Vec<Media>,
}

impl From<rgb_lib::wallet::AssetCFA> for AssetRgb25 {
    fn from(origin: rgb_lib::wallet::AssetCFA) -> AssetRgb25 {
        AssetRgb25 {
            asset_id: origin.asset_id,
            name: origin.name,
            precision: origin.precision,
            description: origin.description,
            balance: Balance::from(origin.balance),
            data_paths: origin
                .data_paths
                .into_iter()
                .map(Media::from)
                .collect::<Vec<Media>>(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Assets {
    rgb20: Option<Vec<AssetRgb20>>,
    rgb25: Option<Vec<AssetRgb25>>,
}

impl From<rgb_lib::wallet::Assets> for Assets {
    fn from(x: rgb_lib::wallet::Assets) -> Assets {
        Assets {
            rgb20: x.nia.map(|vec| {
                vec.into_iter()
                    .map(AssetRgb20::from)
                    .collect::<Vec<AssetRgb20>>()
            }),
            rgb25: x.cfa.map(|vec| {
                vec.into_iter()
                    .map(AssetRgb25::from)
                    .collect::<Vec<AssetRgb25>>()
            }),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AssetsResult {
    assets: Assets,
}

#[put("/wallet/assets")]
pub async fn put(
    params: web::Json<AssetsParams>,
    data: web::Data<Mutex<ShiroWallet>>,
) -> impl Responder {
    if data.lock().unwrap().wallet.is_some() {
        match actix_web::rt::task::spawn_blocking(move || {
            data.lock()
                .unwrap()
                .wallet
                .as_mut()
                .unwrap()
                .list_assets(params.filter_asset_types.clone())
        })
        .await
        .unwrap()
        {
            Ok(assets) => HttpResponse::Ok().json(AssetsResult {
                assets: Assets::from(assets),
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
                .service(put)
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
            let params = AssetsParams {
                filter_asset_types: vec![],
            };
            let req = test::TestRequest::put()
                .uri("/wallet/assets")
                .set_json(params)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("{:?}", resp);
            assert!(resp.status().is_success());
        }
    }
}
