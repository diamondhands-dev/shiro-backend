use actix_web::{App, HttpServer};

mod healthz;
mod keys;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(healthz::get).service(keys::put))
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
