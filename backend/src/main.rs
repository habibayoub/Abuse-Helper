mod common;
mod models;
mod postgres;
mod routes;

use actix_web::{get, web, App, HttpResponse, HttpServer};
// use std::{thread, time::Duration};
// use actix_web_middleware_keycloak_auth::{AlwaysReturnPolicy, DecodingKey, KeycloakAuth, Role};
use dotenv::dotenv;
use routes::{customer, email, nctns, util};

/// GET /status endpoint to check if the server is alive
#[get("status")]
async fn api_status() -> HttpResponse {
    // Return a 200 OK response with a simple message
    HttpResponse::Ok().json("alive")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load start-up environment variables and set the log level
    dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web_middleware_keycloak_auth=trace");
    env_logger::init();

    // Create the database pool and run migrations
    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    // Create a thread to poll the email server for new emails
    // actix_rt::spawn(async {
    //     loop {
    //         email::poll().await;
    //         tokio::time::sleep(Duration::from_secs(60)).await;
    //     }
    // });

    // Start the Actix server
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    HttpServer::new(move || {
        // Create the Keycloak authentication middleware
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

        // Create the Actix application
        App::new()
            .app_data(web::Data::new(pg_pool.clone())) // Add the database pool to the application
            .service(api_status)
            .service(
                web::scope("/customer")
                    // .wrap(keycloak_auth) // Enable Keycloak authentication
                    .service(customer::list)
                    .service(customer::find),
            )
            .service(
                web::scope("/nctns")
                    // .wrap(keycloak_auth)
                    .service(nctns::list),
            )
            .service(web::scope("/util").service(util::whois))
            .service(web::scope("/email").service(email::send))
    })
    .bind(&address)? // Bind the server to the address
    .run() // Run the server
    .await // Wait for the server to stop
}
