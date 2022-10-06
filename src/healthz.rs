use actix_web::{get, Responder};

#[get("/healthz")]
pub async fn get() -> impl Responder {
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body, body::MessageBody as _, rt::pin, test, web, App};

    #[actix_web::test]
    async fn test_healthz() {
        let app = test::init_service(App::new().service(get)).await;
        let req = test::TestRequest::get().uri("/healthz").to_request();

        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);

        assert!(resp.status().is_success());
    }
}
