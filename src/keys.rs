use actix_web::{put, web, HttpResponse, Responder};
//use actix_web::{get, put, web, HttpResponse, Responder};
use rgb_lib::keys::generate_keys;
use serde::Deserialize;
use serde::Serialize;

use rgb_lib::BitcoinNetwork;

#[derive(Serialize, Deserialize)]
pub struct KeyGenParams {
    network: String,
}

#[derive(Serialize, Deserialize)]
struct KeyGenResult {
    mnemonic: String,
    xpub: String,
    xpub_fingerprint: String,
}

//#[get("/keys")]
//pub async fn get() -> impl Responder {
//    ""
//}

#[put("/keys")]
pub async fn put(params: web::Json<KeyGenParams>) -> impl Responder {
    let network = if params.network == *"mainnet" {
        BitcoinNetwork::Mainnet
    } else if params.network == *"testnet" {
        BitcoinNetwork::Testnet
    } else if params.network == *"regtest" {
        BitcoinNetwork::Regtest
    } else if params.network == *"signet" {
        BitcoinNetwork::Signet
    } else {
        return HttpResponse::BadRequest().finish();
    };

    let keys = generate_keys(network);
    let result = KeyGenResult {
        mnemonic: keys.mnemonic,
        xpub: keys.xpub,
        xpub_fingerprint: keys.xpub_fingerprint,
    };
    HttpResponse::Ok().json(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{
        body,
        body::MessageBody as _,
        http::{self, header::ContentType},
        rt::pin,
        test::{self, read_body_json},
        web, App,
    };

    #[actix_web::test]
    async fn get() {
        //        let app = test::init_service(App::new().service(get)).await;
        //        let req = test::TestRequest::get().uri("/get").to_request();
        //
        //        let resp = test::call_service(&app, req).await;
        //        println!("{:?}", resp);
        //
        //        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_put() {
        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put().uri("/keys").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_put_mainnet() {
        let payload = KeyGenParams {
            network: "mainnet".to_string(),
        };

        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let result: KeyGenResult = read_body_json(resp).await;
    }

    #[actix_web::test]
    async fn test_put_testnet() {
        let payload = KeyGenParams {
            network: "testnet".to_string(),
        };

        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let result: KeyGenResult = read_body_json(resp).await;
    }

    #[actix_web::test]
    async fn test_put_regtest() {
        let payload = KeyGenParams {
            network: "regtest".to_string(),
        };

        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let result: KeyGenResult = read_body_json(resp).await;
    }

    #[actix_web::test]
    async fn test_put_signet() {
        let payload = KeyGenParams {
            network: "signet".to_string(),
        };

        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let result: KeyGenResult = read_body_json(resp).await;
    }
}
