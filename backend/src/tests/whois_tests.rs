use super::common;
use crate::models::auth::{LoginForm, TokenResponse};
use crate::routes::auth;
use crate::util;
use actix_web::http::StatusCode;
use actix_web::{test, web, App};

#[actix_rt::test]
async fn test_whois_domain() {
    common::initialize_tests().await;
    let pool = common::get_db_pool().clone();

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/auth").service(auth::login))
            .service(web::scope("/util").service(util::whois)),
    )
    .await;

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

    let request = test::TestRequest::get()
        .uri("/util/whois/google.com")
        .insert_header((
            "Authorization",
            format!("Bearer {}", login_resp.access_token),
        ))
        .to_request();

    let resp = test::call_service(&mut app, request).await;
    println!("Response status: {:?}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_whois_ip() {
    common::initialize_tests().await;
    let pool = common::get_db_pool().clone();

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/auth").service(auth::login))
            .service(web::scope("/util").service(util::whois)),
    )
    .await;

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

    let request = test::TestRequest::get()
        .uri("/util/whois/8.8.8.8")
        .insert_header((
            "Authorization",
            format!("Bearer {}", login_resp.access_token),
        ))
        .to_request();

    let resp = test::call_service(&mut app, request).await;
    println!("Response status: {:?}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);
}
