use crate::wallet::WalletState;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct WalletDir {
    wallet_dir: String,
}

#[get("/wallet/dir")]
pub async fn get(arc: web::Data<Arc<RwLock<WalletState>>>) -> impl Responder {
    if let Ok(wallet_state) = arc.write() {
        if let Some(wallet_data) = &wallet_state.wallet_data {
            HttpResponse::Ok().json(WalletDir {
                wallet_dir: wallet_data.data_dir.clone(),
            })
        } else {
            HttpResponse::BadRequest().body("")
        }
    } else {
        HttpResponse::BadRequest().body("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};

    #[actix_web::test]
    async fn test_get_failed() {
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
                .service(get)
                .service(crate::wallet::put),
        )
        .await;
        let wallet_params = crate::wallet::WalletParams {
            mnemonic: "".to_string(),
            pubkey: "".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert_eq!(wallet_resp.status(), http::StatusCode::BAD_REQUEST);

        let req = test::TestRequest::get().uri("/wallet/dir").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(wallet_resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_get() {
        let wallet_state = Arc::new(RwLock::new(WalletState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(wallet_state.clone()))
                .service(get)
                .service(crate::wallet::put),
        )
        .await;
        let wallet_params = crate::wallet::WalletParams {
            mnemonic: "save call film frog usual market noodle hope stomach chat word worry".to_string(),
            pubkey: "xpub661MyMwAqRbcGexM5um6FYobDPjNH1tmWjxhDkbhfHfxvNpdsmhnvzCDGfemmmNLagBTSSno9nxvaknvDDvqux8sQqrfGPGzFc2JKnf4KL9".to_string(),
        };
        let wallet_req = test::TestRequest::put()
            .uri("/wallet")
            .set_json(wallet_params)
            .to_request();
        let wallet_resp = test::call_service(&app, wallet_req).await;
        println!("{:?}", wallet_resp);
        assert!(wallet_resp.status().is_success());

        let req = test::TestRequest::get().uri("/wallet/dir").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(wallet_resp.status().is_success());
    }
}
