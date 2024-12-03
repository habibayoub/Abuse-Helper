mod auth;
mod llm;
mod middleware;
mod models;
mod postgres;
mod routes;

use crate::models::es::ESClient;
use crate::postgres::run_migrations;
use actix_web::{web, App, HttpServer};
use deadpool_postgres::Pool;
use env_logger::Env;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use log;
use middleware::Logger;

/// Populates the system with test email data
///
/// # Test Data Categories
/// - Security alerts
/// - Phishing attempts
/// - Malware notifications
/// - DDoS reports
///
/// # Returns
/// * `Result<(), String>` - Success or error message
async fn populate_test_emails() -> Result<(), String> {
    // Get the SMTP server from the environment
    let smtp_server = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
    let smtp_port = std::env::var("SMTP_PORT").unwrap_or_else(|_| "3025".to_string());

    // Create the mailer
    let mailer =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_server)
            .port(smtp_port.parse::<u16>().unwrap_or(3025))
            .build();

    // Create the test email templates
    let templates = vec![
        (
            "Account Security Alert",
            "Your account has been temporarily suspended. Click here to verify your identity: http://suspicious-link.com",
            "security@totally-legitimate-bank.com"
        ),
        (
            "Urgent: Payment Required",
            "Dear valued customer,\n\nWe noticed an outstanding payment of $499.99 for your recent order.\nPlease process the payment within 24 hours to avoid account suspension.\n\nClick here to pay: http://fake-payment.com",
            "billing@amaz0n-security.com"
        ),
        (
            "Lottery Winner Notification",
            "Congratulations! You've won $1,000,000 in our international lottery.\nTo claim your prize, please send us your banking details and a processing fee of $100.",
            "lottery@international-prizes.com"
        ),
        (
            "IT Department: Password Reset Required",
            "Your company email password will expire in 24 hours.\nClick here to reset: http://fake-corporate-login.com\n\nIT Department",
            "it-support@corpor8-system.com"
        ),
        (
            "Malware Detection Alert",
            "We detected malicious software (TrojanRAT.exe) attempting to connect to IP: 192.168.1.100. Multiple connection attempts from botnet command server at 45.67.89.123.",
            "security@network-monitor.com"
        ),
        (
            "DDoS Attack Report",
            "Ongoing DDoS attack detected. Source IPs: 23.45.67.89, 98.76.54.32. Target: web-server-1. Peak traffic: 50Gbps.",
            "alerts@ddos-protection.com"
        ),
    ];

    // Iterate over the test email templates
    for (subject, body, from) in templates {
        // Parse the from address
        let from_address = from.parse::<Mailbox>().map_err(|e| e.to_string())?;
        // Parse the to address
        let to_address = "test@localhost"
            .parse::<Mailbox>()
            .map_err(|e| e.to_string())?;

        // Create the email message
        let email = Message::builder()
            .from(from_address)
            .to(to_address)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| e.to_string())?;

        // Send the email
        match mailer.send(email).await {
            Ok(_) => log::info!("Sent test email: {}", subject),
            Err(e) => log::error!("Failed to send test email {}: {}", subject, e),
        }
    }

    Ok(())
}

/// Initializes ElasticSearch indices and mappings
///
/// # Indices Created
/// - emails: Email document storage
/// - tickets: Ticket document storage
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
///
/// # Index Configuration
/// - Creates indices if not exist
/// - Updates mappings if needed
/// - Ensures proper analyzers
async fn init_elasticsearch() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing ElasticSearch indices...");
    // Create the ElasticSearch client
    let client = ESClient::new().await?;

    log::info!("Creating/updating email index...");
    // Create the email index
    client.ensure_index("emails").await?;

    log::info!("Creating/updating ticket index...");
    // Create the ticket index
    client.ensure_index("tickets").await?;

    log::info!("ElasticSearch indices initialized successfully");
    Ok(())
}

/// Performs database cleanup operations
///
/// # Cleanup Steps
/// 1. Removes email-ticket associations
/// 2. Deletes email records
/// 3. Deletes ticket records
/// 4. Resets ElasticSearch indices
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
async fn cleanup_database(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Cleaning up database...");
    // Get a client from the pool
    let client = pool.get().await?;

    // Delete in correct order to handle foreign key constraints
    client.execute("DELETE FROM email_tickets", &[]).await?;
    client.execute("DELETE FROM emails", &[]).await?;
    client.execute("DELETE FROM tickets", &[]).await?;

    // Delete and recreate the Elasticsearch indices
    let es_client = ESClient::new().await?;

    // Delete indices if they exist
    let _ = es_client
        .delete_index("emails")
        .await
        .map_err(|e| log::warn!("Failed to delete emails index: {}", e));

    // Delete the tickets index if it exists
    let _ = es_client
        .delete_index("tickets")
        .await
        .map_err(|e| log::warn!("Failed to delete tickets index: {}", e));

    // Create indices with proper mappings
    es_client.ensure_index("emails").await?;
    es_client.ensure_index("tickets").await?;

    log::info!("Database cleanup completed");
    Ok(())
}

/// Application entry point
///
/// # Initialization Steps
/// 1. Configures logging
/// 2. Sets up database pool
/// 3. Runs migrations
/// 4. Performs cleanup
/// 5. Populates test data
/// 6. Initializes ElasticSearch
/// 7. Starts HTTP server
///
/// # Server Configuration
/// - Uses environment variable `ADDRESS` for binding
/// - Configures routes and middleware
/// - Sets up connection pools
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging with filter for tokenizers warnings
    std::env::set_var("RUST_LOG", "info");

    // Initialize logging
    env_logger::Builder::from_env(Env::default())
        .format_timestamp_millis()
        .init();

    // Create the database pool and run migrations
    let pg_pool = postgres::create_pool();

    // Run migrations
    if let Err(e) = run_migrations(&pg_pool).await {
        log::error!("Failed to run migrations: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed",
        ));
    }

    // Clean up database
    if let Err(e) = cleanup_database(&pg_pool).await {
        log::error!("Failed to clean up database: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Database cleanup failed",
        ));
    }

    // Populate test emails
    if let Err(e) = populate_test_emails().await {
        log::error!("Failed to populate test emails: {}", e);
    }

    // Initialize ElasticSearch
    if let Err(e) = init_elasticsearch().await {
        log::error!("Failed to initialize ElasticSearch: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ElasticSearch initialization failed",
        ));
    }

    // Start the Actix server
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    // Create the HTTP server
    HttpServer::new(move || {
        App::new()
            // Add the database pool to the app data
            .app_data(web::Data::new(pg_pool.clone()))
            // Add the logger middleware
            .wrap(Logger::new())
            // Configure the routes
            .service(routes::config::configure_routes())
    })
    // Bind the server to the address
    .bind(&address)?
    // Run the server
    .run()
    .await
}
