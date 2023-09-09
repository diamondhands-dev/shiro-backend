use actix_web::{put, web, HttpResponse, Responder};
use rgb_lib::wallet::{Invoice, InvoiceData};
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct RgbInvoice {
    invoice_string: String,
}

#[put("/wallet/invoice")]
pub async fn put(params: web::Json<RgbInvoice>) -> impl Responder {
    let decoded = Invoice::new(params.invoice_string.clone());
    match decoded {
        Ok(invoice) => HttpResponse::Ok().json(InvoiceData {
            asset_iface: invoice.invoice_data().asset_iface,
            blinded_utxo: invoice.invoice_data().blinded_utxo,
            asset_id: invoice.invoice_data().asset_id,
            amount: invoice.invoice_data().amount,
            expiration_timestamp: invoice.invoice_data().expiration_timestamp,
            transport_endpoints: invoice.invoice_data().transport_endpoints,
        }),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{http, test, App};
    use rgb_lib::wallet::BlindedUTXO;

    #[actix_web::test]
    async fn test_put_invoice() {
        let app = test::init_service(App::new().service(put)).await;
        let payload = RgbInvoice {
            invoice_string: ("rgb:9usESnQYgX2KWNycD3cYRGddBc65uDC6gPeHjV9XzbHU/RGB20/10+DLrwJdhSdhhhxrUGZudY6C6ubbdPn14SZ1FLuTT3nUER?expiry=1694222774&endpoints=rpc://127.0.0.1:3000/json-rpc")
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
        let result = BlindedUTXO::new(body.blinded_utxo);
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_put_with_bad_request() {
        let app = test::init_service(App::new().service(put)).await;
        let payload = RgbInvoice {
            invoice_string: ("helloRGB").to_string(),
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
