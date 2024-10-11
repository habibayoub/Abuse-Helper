mod auth;
mod llm;
mod middleware;
mod models;
mod postgres;
mod routes;

use crate::postgres::run_migrations;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use middleware::{Auth, Logger};
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

    // Run migrations
    if let Err(e) = run_migrations(&pg_pool).await {
        eprintln!("Failed to run migrations: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed",
        ));
    }

    // Start the Actix server
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    HttpServer::new(move || {
        // Create the Actix application
        App::new()
            .app_data(web::Data::new(pg_pool.clone())) // Add the database pool to the application
            .wrap(Logger::new())
            .service(api_status)
            .service(
                web::scope("/auth")
                    .service(routes::auth::login)
                    .service(routes::auth::refresh)
                    .service(routes::auth::exchange_token),
            )
            .service(
                web::scope("")
                    .wrap(Auth::new())
                    .service(web::scope("/auth").service(routes::auth::logout))
                    .service(
                        web::scope("/customer")
                            .wrap(Auth::new().role("admin"))
                            .service(customer::list)
                            .service(customer::find),
                    )
                    .service(
                        web::scope("/nctns")
                            .wrap(Auth::new().role("user"))
                            .service(nctns::list),
                    )
                    .service(
                        web::scope("/util")
                            .wrap(Auth::new().role("user"))
                            .service(util::whois),
                    )
                    .service(
                        web::scope("/email")
                            .wrap(Auth::new().role("admin"))
                            .service(email::send),
                    ),
            )
    })
    .bind(&address)?
    .run()
    .await
}
