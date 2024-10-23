use super::common;
use crate as abuse_helper;
use abuse_helper::models::customer::Customer;
use abuse_helper::routes::customer;
use actix_web::{test, web, App};
use serde_json;

#[actix_rt::test]
async fn test_list_customers() {
    let pool = common::get_db_pool().clone();

    // Clear the customers table before the test
    common::clear_table(&pool, "customers")
        .await
        .expect("Failed to clear customers table");

    // Insert test data
    let client = pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO customers (email, first_name, last_name, ip) VALUES ($1, $2, $3, $4)",
            &[
                &"john.smith@gmail.com".to_string(),
                &"John".to_string(),
                &"Smith".to_string(),
                &"192.0.0.1".to_string(),
            ],
        )
        .await
        .expect("Failed to insert test customer");

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/customer").service(customer::list)),
    )
    .await;

    let req = test::TestRequest::get().uri("/customer").to_request();
    let resp = test::call_service(&mut app, req).await;
    println!("Response status: {:?}", resp.status());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert body to string");
    println!("Response body: {}", body_str);

    let parsed_resp: Result<Vec<Customer>, serde_json::Error> = serde_json::from_str(&body_str);
    match parsed_resp {
        Ok(customers) => {
            assert!(!customers.is_empty());
            assert_eq!(customers[0].email, "john.smith@gmail.com");
            assert_eq!(customers[0].ip, Some("192.0.0.1".to_string()));
        }
        Err(e) => panic!(
            "Failed to parse response: {:?}\nResponse body: {}",
            e, body_str
        ),
    }
}
