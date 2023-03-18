use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Invoice, InvoiceData};
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct Bech32Invoice {
    bech32_invoice: String,
}

#[put("/wallet/invoice")]
pub async fn put(params: web::Json<Bech32Invoice>) -> impl Responder {
    let decoded = Invoice::new(params.bech32_invoice.clone());
    match decoded {
        Ok(invoice) => HttpResponse::Ok().json(InvoiceData {
            blinded_utxo: invoice.invoice_data().blinded_utxo,
            asset_id: invoice.invoice_data().asset_id,
            amount: invoice.invoice_data().amount,
            expiration_timestamp: invoice.invoice_data().expiration_timestamp,
            consignment_endpoints: invoice.invoice_data().consignment_endpoints,
        }),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{http, test, App};

    #[actix_web::test]
    async fn test_put() {
        let app = test::init_service(App::new().service(put)).await;
        let payload = Bech32Invoice {
            bech32_invoice: ("i1q93kqcyuklwuz3z87zwudx28tutzduwg0jd8tmz6vpt2pcxm5h8h7llntcek2vsnqvesxprsxz7pzc6wqxcm3gpztgxgcryv4gxpjff9qhz4d7h6q4zlj9v402v5txw9ukynjwdfy4avn7delfvut7tehfzstjgqwhe0f0")
                .to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/invoice")
            .set_json(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
        let body: InvoiceData = test::read_body_json(resp).await;
        assert!(body.blinded_utxo.starts_with("txo"));
    }

    #[actix_web::test]
    async fn test_put_with_bad_request() {
        let app = test::init_service(App::new().service(put)).await;
        let payload = Bech32Invoice {
            bech32_invoice: ("helloRGB").to_string(),
        };
        let req = test::TestRequest::put()
            .uri("/wallet/invoice")
            .set_json(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}
