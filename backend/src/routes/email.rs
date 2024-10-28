use crate::models::{
    email::{Email, OutgoingEmail},
    ticket::TicketType,
};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use imap::Session;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use mailparse::MailHeaderMap;
use native_tls::TlsConnector;
use regex::Regex;
use std::env;
use uuid::Uuid;
use reqwest;
use serde_json::Value;

/// Function to send an email using SMTP
async fn send_email(email: OutgoingEmail) -> Result<String, String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "mailcrab".to_string());
    let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "1025".to_string());
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

    let recipient = email.recipient.email.clone();
    let from_address = smtp_username
        .parse::<Mailbox>()
        .map_err(|e| e.to_string())?;

    let email_payload = Message::builder()
        .from(from_address)
        .to(email.recipient)
        .subject(&email.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(email.body)
        .map_err(|e| e.to_string())?;

    let mailer =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_server)
            .port(smtp_port.parse::<u16>().unwrap_or(1025))
            .build();

    mailer
        .send(email_payload)
        .await
        .map(|_| format!("Successfully sent {} to {}", email.subject, recipient))
        .map_err(|e| format!("Could not send email: {:#?}", e))
}

/// POST /email/send endpoint
#[post("/send")]
pub async fn send(pool: web::Data<Pool>, email: web::Json<OutgoingEmail>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let email_data = email.into_inner();
    let recipient_email = email_data.recipient.email.to_string();
    let subject = email_data.subject.clone();
    let body = email_data.body.clone();

    // First send the email
    match send_email(email_data).await {
        Ok(result) => {
            // Store the sent email
            let stmt = match client
                .prepare(
                    "INSERT INTO emails (id, from, to, subject, body, received_at) 
                     VALUES ($1, $2, $3, $4, $5, $6) 
                     RETURNING id",
                )
                .await
            {
                Ok(stmt) => stmt,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };

            let id = Uuid::new_v4().to_string();
            let smtp_username =
                env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

            match client
                .query_one(
                    &stmt,
                    &[
                        &id,
                        &smtp_username,
                        &vec![recipient_email],
                        &subject,
                        &body,
                        &Utc::now(),
                    ],
                )
                .await
            {
                Ok(_) => HttpResponse::Created().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

/// Function to extract IP addresses from text
fn extract_ips(text: &str) -> Vec<String> {
    let ip_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    ip_regex
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Function to determine ticket type from email content
fn determine_ticket_type(subject: &str, body: &str) -> TicketType {
    let text = format!("{} {}", subject, body).to_lowercase();

    if text.contains("malware") || text.contains("virus") || text.contains("trojan") {
        TicketType::Malware
    } else if text.contains("phishing") || text.contains("credential") || text.contains("login") {
        TicketType::Phishing
    } else if text.contains("scam") || text.contains("fraud") {
        TicketType::Scam
    } else if text.contains("spam") {
        TicketType::Spam
    } else {
        TicketType::Other
    }
}

/// Function to create a ticket from email content
async fn create_ticket_from_email(pool: &Pool, email: &Email) -> Result<Uuid, String> {
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let ticket_type = determine_ticket_type(&email.subject, &email.body);
    let ip_addresses = extract_ips(&email.body);
    let ip_address = ip_addresses.first().map(|s| s.to_string());
    let ticket_id = Uuid::new_v4();

    let stmt = client
        .prepare(
            "INSERT INTO tickets (id, ticket_type, ip_address, email_id, subject, description) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id",
        )
        .await
        .map_err(|e| e.to_string())?;

    client
        .query_one(
            &stmt,
            &[
                &ticket_id,
                &ticket_type.to_string(),
                &ip_address,
                &email.id,
                &email.subject,
                &email.body,
            ],
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ticket_id)
}

async fn fetch_emails(pool: &Pool) -> Result<Vec<Email>, String> {
    let mailcrab_url = env::var("MAILCRAB_URL")
        .unwrap_or_else(|_| "http://mailcrab:1080".to_string());
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/messages", mailcrab_url))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from Mailcrab API: {}", e))?;

    let messages: Vec<Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Mailcrab response: {}", e))?;

    let mut emails = Vec::new();
    
    for message in messages {
        let email = Email {
            id: message["ID"].as_str().unwrap_or("").to_string(),
            from: message["From"].as_str().unwrap_or("Unknown").to_string(),
            to: vec![message["To"].as_str().unwrap_or("").to_string()],
            subject: message["Subject"].as_str().unwrap_or("No Subject").to_string(),
            body: message["Text"].as_str().unwrap_or("").to_string(),
            received_at: DateTime::parse_from_rfc3339(
                message["Created"].as_str().unwrap_or(&Utc::now().to_rfc3339())
            )
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc),
        };

        // Create ticket for the email
        if let Err(e) = create_ticket_from_email(pool, &email).await {
            eprintln!("Failed to create ticket for email {}: {}", email.id, e);
        }

        emails.push(email);
    }

    Ok(emails)
}

/// GET /email/list endpoint
#[get("/list")]
pub async fn list_emails(pool: web::Data<Pool>) -> HttpResponse {
    match fetch_emails(&pool).await {
        Ok(emails) => HttpResponse::Ok().json(emails),
        Err(e) => {
            HttpResponse::InternalServerError().json(format!("Failed to fetch emails: {}", e))
        }
    }
}

/// Function to poll for new emails
pub async fn poll(pool: Pool) {
    match fetch_emails(&pool).await {
        Ok(emails) => {
            if !emails.is_empty() {
                println!("Found {} new emails", emails.len());
                for email in emails {
                    println!(
                        "Email from: {}, subject: {}, received at: {}",
                        email.from, email.subject, email.received_at
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to poll emails: {}", e),
    }
}
