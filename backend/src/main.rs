mod common;
mod llm;
mod models;
mod postgres;
mod routes;

use actix_web::{get, web, App, HttpResponse, HttpServer};
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
    std::env::set_var("RUST_LOG", "info");
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
        // Create the Actix application
        App::new()
            .app_data(web::Data::new(pg_pool.clone())) // Add the database pool to the application
            .service(api_status)
            .service(
                web::scope("/customer")
                    .service(customer::list)
                    .service(customer::find),
            )
            .service(web::scope("/nctns").service(nctns::list))
            .service(web::scope("/util").service(util::whois))
            .service(web::scope("/email").service(email::send))
    })
    .bind(&address)? // Bind the server to the address
    .run() // Run the server
    .await // Wait for the server to stop
}
