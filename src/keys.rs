use actix_web::{get, put, web, HttpResponse, Responder};
use rgb_lib::keys::generate_keys;
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

#[get("/keys")]
pub async fn get() -> impl Responder {
    ""
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
        body,
        body::MessageBody as _,
        http::{self, header::ContentType},
        rt::pin,
        test::{self, read_body_json},
        web, App,
    };

    #[actix_web::test]
    async fn test_get() {
        let app = test::init_service(App::new().service(get)).await;
        let req = test::TestRequest::get().uri("/keys").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
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
