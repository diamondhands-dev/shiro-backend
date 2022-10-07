use actix_web::{get, App, HttpServer, Responder};

#[get("/healthz")]
async fn healthz() -> impl Responder {
    "healthy"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(healthz))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
