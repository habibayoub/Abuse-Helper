use super::common;
use crate::models::customer::LookUpForm;
use crate::{self as abuse_helper, middleware::Auth};
use abuse_helper::models::auth::{LoginForm, TokenResponse};
use abuse_helper::models::customer::Customer;
use abuse_helper::routes::{auth, customer};
use actix_web::{test, web, App};
use serde_json;

#[actix_rt::test]
async fn test_list_customers() {
    common::initialize_tests().await;
    let pool = common::get_db_pool().clone();

    // Create test app
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/auth").service(auth::login))
            .service(
                web::scope("")
                    .wrap(Auth::new())
                    .service(web::scope("/customer").service(customer::list)),
            ),
    )
    .await;

    // Login as admin
    let login_form = LoginForm {
        email: "admin@example.com".to_string(),
        password: "admin123".to_string(),
    };

    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_form)
        .to_request();

    let login_resp: TokenResponse = test::call_and_read_body_json(&mut app, login_req).await;

    println!("Login response: {:?}", login_resp.access_token);

    // Make authenticated request to list customers
    let req = test::TestRequest::get()
        .uri("/customer/list")
        .insert_header((
            "Authorization",
            format!("Bearer {}", login_resp.access_token),
        ))
        .to_request();

    println!("Request headers: {:?}", req.head());

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

#[actix_rt::test]
async fn test_find_customer() {
    common::initialize_tests().await;
    let pool = common::get_db_pool().clone();

    // Create test app
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/auth").service(auth::login))
            .service(
                web::scope("")
                    .wrap(Auth::new())
                    .service(web::scope("/customer").service(customer::find)),
            ),
    )
    .await;

    // Login as admin
    let login_form = LoginForm {
        email: "admin@example.com".to_string(),
        password: "admin123".to_string(),
    };

    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_form)
        .to_request();

    let login_resp: TokenResponse = test::call_and_read_body_json(&mut app, login_req).await;

    println!("Login response: {:?}", login_resp.access_token);

    // Prepare the request body
    let form = LookUpForm {
        email: Some("john.smith@gmail.com".to_string()),
        ip: None,
        uuid: None,
    };
    let req = test::TestRequest::post()
        .uri("/customer/find")
        .set_json(&form)
        .insert_header((
            "Authorization",
            format!("Bearer {}", login_resp.access_token),
        ))
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    println!("Response status: {:?}", resp.status());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert body to string");
    println!("Response body: {}", body_str);

    let parsed_resp: Result<Customer, serde_json::Error> = serde_json::from_str(&body_str);

    match parsed_resp {
        Ok(customer) => {
            assert_eq!(customer.email, "john.smith@gmail.com");
        }
        Err(e) => panic!(
            "Failed to parse response: {:?}\nResponse body: {}",
            e, body_str
        ),
    }
}
