use actix_web::{web, App, HttpServer};
use std::sync::{Arc, RwLock};

mod healthz;
mod keys;
mod wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet_state = Arc::new(RwLock::new(wallet::WalletState::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wallet_state.clone()))
            .service(healthz::get)
            .service(keys::post)
            .service(keys::put)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body, body::MessageBody as _, rt::pin, test, web, App};

    #[actix_web::test]
    async fn test_root() {
        let app = test::init_service(App::new()).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
    }
}
