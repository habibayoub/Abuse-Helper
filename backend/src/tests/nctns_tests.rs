use super::common;
use crate as abuse_helper;
use abuse_helper::models::nctns::NCTNS;
use abuse_helper::routes::nctns;
use actix_web::{test, web, App};
use serde_json;

#[actix_rt::test]
async fn test_list_nctns() {
    let pool = common::get_db_pool().clone();

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/nctns").service(nctns::list)),
    )
    .await;

    let req = test::TestRequest::get().uri("/nctns/list").to_request();
    let resp = test::call_service(&mut app, req).await;

    println!("Response status: {:?}", resp.status());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert body to string");
    println!("Response body: {}", body_str);

    let parsed_resp: Result<Vec<NCTNS>, serde_json::Error> = serde_json::from_str(&body_str);

    match parsed_resp {
        Ok(nctns_list) => {
            assert!(!nctns_list.is_empty());
            assert_eq!(nctns_list[0].source_name, "SourceName1");
            assert_eq!(nctns_list[0].ip, "192.168.1.1");
            assert_eq!(nctns_list[0].domain_name, "example.com");
        }
        Err(e) => {
            panic!(
                "Failed to parse response: {:?}\nResponse body: {}",
                e, body_str
            );
        }
    }
}
