mod auth;
mod llm;
mod middleware;
mod models;
mod postgres;
mod routes;

use crate::postgres::run_migrations;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use middleware::Logger;

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
            .service(routes::config::configure_routes())
    })
    .bind(&address)?
    .run()
    .await
}
