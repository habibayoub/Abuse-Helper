mod auth;
mod llm;
mod middleware;
mod models;
mod postgres;
mod routes;

use crate::postgres::run_migrations;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use log;
use middleware::Logger;

async fn populate_test_emails() -> Result<(), String> {
    let smtp_server = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
    let smtp_port = std::env::var("SMTP_PORT").unwrap_or_else(|_| "3025".to_string());

    let mailer =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_server)
            .port(smtp_port.parse::<u16>().unwrap_or(3025))
            .build();

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

    for (subject, body, from) in templates {
        let from_address = from.parse::<Mailbox>().map_err(|e| e.to_string())?;
        let to_address = "test@localhost"
            .parse::<Mailbox>()
            .map_err(|e| e.to_string())?;

        let email = Message::builder()
            .from(from_address)
            .to(to_address)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| e.to_string())?;

        match mailer.send(email).await {
            Ok(_) => log::info!("Sent test email: {}", subject),
            Err(e) => log::error!("Failed to send test email {}: {}", subject, e),
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging with filter for tokenizers warnings
    std::env::set_var("RUST_LOG", "info");
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

    // Populate test emails
    if let Err(e) = populate_test_emails().await {
        log::error!("Failed to populate test emails: {}", e);
    }

    // Start the Actix server
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            .wrap(Logger::new())
            .service(routes::config::configure_routes())
    })
    .bind(&address)?
    .run()
    .await
}
