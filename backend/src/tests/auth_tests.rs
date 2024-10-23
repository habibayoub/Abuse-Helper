use super::common;
use crate::models::auth::{LoginForm, TokenResponse};
use crate::routes::auth;
use actix_web::{test, web, App};

#[actix_rt::test]
async fn test_login() {
    let pool = common::get_db_pool().clone();

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/auth").service(auth::login)),
    )
    .await;

    let login_form = LoginForm {
        email: "admin@example.com".to_string(),
        password: "admin123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_form)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    println!("Response status: {:?}", resp.status());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert body to string");
    println!("Response body: {}", body_str);

    let parsed_resp: Result<TokenResponse, serde_json::Error> = serde_json::from_str(&body_str);
    match parsed_resp {
        Ok(token_response) => {
            assert!(!token_response.access_token.is_empty());
            assert!(token_response.refresh_token.is_some());
            assert_eq!(token_response.user.email, "admin@example.com");
        }
        Err(e) => panic!(
            "Failed to parse response: {:?}\nResponse body: {}",
            e, body_str
        ),
    }
}
