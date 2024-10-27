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

/// Function to send an email using SMTP
async fn send_email(email: OutgoingEmail) -> Result<String, String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
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

fn connect_imap() -> Result<Session<native_tls::TlsStream<std::net::TcpStream>>, String> {
    let imap_server = env::var("IMAP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
    let imap_port = env::var("IMAP_PORT").unwrap_or_else(|_| "1143".to_string());
    let username = env::var("IMAP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());
    let password = env::var("IMAP_PASSWORD").unwrap_or_else(|_| "password".to_string());

    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::connect(
        (
            imap_server.as_str(),
            imap_port.parse::<u16>().unwrap_or(1143),
        ),
        imap_server.as_str(),
        &tls,
    )
    .map_err(|e| format!("Connection error: {}", e))?;

    client
        .login(username, password)
        .map_err(|(e, _)| format!("Login error: {}", e))
}

async fn fetch_emails(pool: &Pool) -> Result<Vec<Email>, String> {
    let mut imap_session = connect_imap()?;

    imap_session
        .select("INBOX")
        .map_err(|e| format!("Failed to select INBOX: {}", e))?;

    let sequence_set = "1:*";
    let messages = imap_session
        .fetch(sequence_set, "(RFC822 INTERNALDATE)")
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    let mut emails = Vec::new();

    for message in messages.iter() {
        if let Some(body) = message.body() {
            if let Ok(parsed) = mailparse::parse_mail(body) {
                let headers = parsed.get_headers();

                // Fix: Use get_header_value instead of get_first_value
                let from = headers
                    .get_first_value("From")
                    .unwrap_or_else(|| "Unknown".to_string());

                let to: Vec<String> = headers.get_all_values("To");

                let subject = headers
                    .get_first_value("Subject")
                    .unwrap_or_else(|| "No Subject".to_string());

                let received_at = message
                    .internal_date()
                    .map(|date| DateTime::<Utc>::from(date))
                    .unwrap_or_else(|| Utc::now());

                let body = parsed
                    .get_body()
                    .map_err(|_| "Failed to get body".to_string())?;

                let email = Email {
                    id: message.message.to_string(),
                    from,
                    to,
                    subject,
                    body,
                    received_at,
                };

                // Create ticket for the email
                if let Err(e) = create_ticket_from_email(pool, &email).await {
                    eprintln!("Failed to create ticket for email {}: {}", email.id, e);
                }

                emails.push(email);
            }
        }
    }

    imap_session.logout().ok();
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
