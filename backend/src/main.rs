mod models;
mod postgres;
mod routes;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use actix_web_middleware_keycloak_auth::{AlwaysReturnPolicy, DecodingKey, KeycloakAuth, Role};
use dotenv::dotenv;
use routes::customer::{find_customer, list_customers};

#[get("status")]
async fn api_status() -> HttpResponse {
    HttpResponse::Ok().json("alive")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web_middleware_keycloak_auth=trace");
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    HttpServer::new(move || {
        // let keycloak_public_key =
        //     std::env::var("KEYCLOAK_PUBLIC_KEY").expect("KEYCLOAK_PUBLIC_KEY must be set.");

        // let keycloak_auth = KeycloakAuth {
        //     detailed_responses: true,
        //     passthrough_policy: AlwaysReturnPolicy,
        //     keycloak_oid_public_key: DecodingKey::from_rsa_pem(keycloak_public_key.as_bytes())
        //         .unwrap(),
        //     required_roles: vec![Role::Realm {
        //         role: "admin".to_owned(),
        //     }],
        // };

        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            .service(api_status)
            .service(
                web::scope("/private")
                    // .wrap(keycloak_auth)
                    .service(list_customers)
                    .service(find_customer),
            )
    })
    .bind(&address)?
    .run()
    .await
}
