use actix_web::{post, put, web, HttpResponse, Responder};
use rgb_lib::keys::{generate_keys, restore_keys};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct KeyGenParams {}

#[derive(Serialize, Deserialize)]
struct KeyGenResult {
    mnemonic: String,
    xpub: String,
    xpub_fingerprint: String,
}

#[derive(Serialize, Deserialize)]
pub struct KeyRestoreParams {
    mnemonic: String,
}

#[post("/keys")]
pub async fn post(params: web::Json<KeyRestoreParams>) -> impl Responder {
    let network = shiro_backend::opts::get_bitcoin_network();
    let result = restore_keys(network, params.mnemonic.clone());
    match result {
        Result::Ok(keys) => HttpResponse::Ok().json(KeyGenResult {
            mnemonic: keys.mnemonic,
            xpub: keys.xpub,
            xpub_fingerprint: keys.xpub_fingerprint,
        }),
        Result::Err(_) => HttpResponse::BadRequest().body("Invalid mnemonic"),
    }
}

#[put("/keys")]
pub async fn put(_params: web::Json<KeyGenParams>) -> impl Responder {
    let network = shiro_backend::opts::get_bitcoin_network();

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
        http,
        test::{self, read_body_json},
        App,
    };

    #[actix_web::test]
    async fn test_post_with_no_json() {
        let app = test::init_service(App::new().service(post)).await;
        let req = test::TestRequest::post().uri("/keys").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_post_with_bad_mnemonic() {
        let app = test::init_service(App::new().service(post)).await;
        let payload = KeyRestoreParams {
            mnemonic: ("save call film frog usual market noodle hope stomach chat word worry bad")
                .to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_post() {
        let app = test::init_service(App::new().service(post)).await;
        let payload = KeyRestoreParams {
            mnemonic: ("save call film frog usual market noodle hope stomach chat word worry")
                .to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let result: KeyGenResult = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            result.mnemonic,
            ("save call film frog usual market noodle hope stomach chat word worry").to_string()
        );
        assert_eq!(result.xpub, "xpub661MyMwAqRbcGexM5um6FYobDPjNH1tmWjxhDkbhfHfxvNpdsmhnvzCDGfemmmNLagBTSSno9nxvaknvDDvqux8sQqrfGPGzFc2JKnf4KL9".to_string());
        assert_eq!(result.xpub_fingerprint, "60ec7707");
    }

    #[actix_web::test]
    async fn test_put() {
        let payload = KeyGenParams {};
        let app = test::init_service(App::new().service(put)).await;
        let req = test::TestRequest::put()
            .uri("/keys")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
        let result: KeyGenResult = read_body_json(resp).await;
        assert_eq!(result.mnemonic.split(' ').count(), 12);
        assert!(result.xpub.starts_with("xpub"));
        assert_ne!(result.xpub_fingerprint, "");
    }
}
