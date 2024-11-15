use crate::{
    llm::analyze_threat,
    models::email::{Email, OutgoingEmail},
};
use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use deadpool_postgres::Pool;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use mailparse::{parse_mail, MailHeaderMap};
use sha2::{Digest, Sha256};
use std::env;
use std::net::TcpStream;
use uuid::Uuid;
/// Function to send an email using SMTP
async fn send_email(email: OutgoingEmail) -> Result<String, String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
    let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "3025".to_string());
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
            .port(smtp_port.parse::<u16>().unwrap_or(3025))
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
                    "INSERT INTO emails (id, sender, recipients, subject, body, received_at) 
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

/// Function to create a ticket from email content
async fn create_ticket_from_email(pool: &Pool, email: &Email) -> Result<Uuid, String> {
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let content = format!(
        "From: {}\nTo: {}\nSubject: {}\nBody: {}",
        email.sender,
        email.recipients.join(", "),
        email.subject,
        email.body
    );

    let analysis = analyze_threat(&content).await?;

    let ip_address = analysis
        .extracted_indicators
        .iter()
        .find(|indicator| indicator.contains('.'))
        .cloned();

    let ticket_id = Uuid::new_v4();

    let enhanced_description = format!(
        "Original Content:\n{}\n\nThreat Analysis:\n- Confidence: {}\n- Identified Threats: {}\n- Extracted Indicators: {}\n\nSummary: {}",
        email.body,
        analysis.confidence_score,
        analysis.identified_threats.join(", "),
        analysis.extracted_indicators.join(", "),
        analysis.summary
    );

    let stmt = client
        .prepare(
            "INSERT INTO tickets (
                id, ticket_type, ip_address, email_id, subject, description,
                confidence_score, identified_threats, extracted_indicators, analysis_summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
            RETURNING id",
        )
        .await
        .map_err(|e| e.to_string())?;

    client
        .query_one(
            &stmt,
            &[
                &ticket_id,
                &analysis.threat_type.to_string(),
                &ip_address,
                &email.id,
                &email.subject,
                &enhanced_description,
                &analysis.confidence_score,
                &analysis.identified_threats,
                &analysis.extracted_indicators,
                &analysis.summary,
            ],
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ticket_id)
}

async fn mark_email_as_analyzed(pool: &Pool, email_id: &str) -> Result<(), String> {
    let client = pool.get().await.map_err(|e| e.to_string())?;

    client
        .execute(
            "UPDATE emails SET analyzed = TRUE WHERE id = $1",
            &[&email_id],
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Modify fetch_emails to only process unanalyzed emails
async fn fetch_emails(pool: &Pool) -> Result<Vec<Email>, String> {
    let imap_server = std::env::var("IMAP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
    let imap_port = std::env::var("IMAP_PORT")
        .unwrap_or_else(|_| "3993".to_string())
        .parse::<u16>()
        .unwrap_or(3993);

    log::info!("IMAP server: {}", imap_server);
    log::info!("IMAP port: {}", imap_port);

    let tls = match native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(tls) => tls,
        Err(e) => return Err(format!("Failed to create TLS connector: {}", e)),
    };

    log::info!("Connecting to IMAP server: {}", imap_server);

    let client = imap::connect((imap_server.clone(), imap_port), imap_server, &tls).unwrap();

    log::info!("Connected to IMAP server");

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = match client.login("test@localhost", "password").map_err(|e| e.0) {
        Ok(imap_session) => imap_session,
        Err(e) => return Err(format!("Failed to login to IMAP: {:?}", e)),
    };

    log::info!("Logged in to IMAP server");

    // Select the INBOX
    let mailbox = imap_session
        .select("INBOX")
        .map_err(|e| format!("Failed to select INBOX: {}", e))?;

    log::info!("Selected INBOX");

    let mut emails = Vec::new();

    // Fetch all messages
    if mailbox.exists > 0 {
        log::info!("INBOX has {} messages", mailbox.exists);

        let sequence_set = format!("1:{}", mailbox.exists);
        let messages = imap_session
            .fetch(sequence_set, "RFC822")
            .map_err(|e| format!("Failed to fetch messages: {}", e))?;

        log::info!("Fetched {} messages", messages.len());

        for message in messages.iter() {
            log::info!("Message: {:?}", message);
            if let Some(body) = message.body() {
                if let Ok(parsed_mail) = parse_mail(body) {
                    log::info!("Parsed mail: {:?}", parsed_mail);
                    let headers = &parsed_mail.headers;
                    let from = headers.get_first_value("From").unwrap_or_default();
                    let to = headers.get_all_values("To");
                    let subject = headers.get_first_value("Subject").unwrap_or_default();
                    let date = headers.get_first_value("Date").unwrap_or_default();

                    // Create a deterministic ID based on email metadata
                    let id_string = format!("{}:{}:{}:{}", from, to.join(","), subject, date);
                    let mut hasher = Sha256::new();
                    hasher.update(id_string.as_bytes());
                    let email_id = format!("{:x}", hasher.finalize());

                    // Check if email has already been analyzed
                    let pg_client = pool.get().await.map_err(|e| e.to_string())?;
                    let exists = pg_client
                        .query_one(
                            "SELECT EXISTS(SELECT 1 FROM emails WHERE id = $1 AND analyzed = TRUE)",
                            &[&email_id],
                        )
                        .await
                        .map_err(|e| e.to_string())?
                        .get::<_, bool>(0);

                    if exists {
                        continue;
                    }

                    let body = parsed_mail.get_body().unwrap_or_default();

                    let email = Email {
                        id: email_id.clone(),
                        sender: from,
                        recipients: to,
                        subject,
                        body,
                        received_at: Utc::now(),
                        analyzed: false,
                    };

                    // Create ticket for the email
                    if let Err(e) = create_ticket_from_email(pool, &email).await {
                        log::error!("Failed to create ticket for email {}: {}", email.id, e);
                    } else {
                        // Mark email as analyzed after successful ticket creation
                        if let Err(e) = mark_email_as_analyzed(pool, &email.id).await {
                            log::error!("Failed to mark email {} as analyzed: {}", email.id, e);
                        } else {
                            log::info!("Created ticket and marked email as analyzed: {}", email.id);
                        }
                    }

                    emails.push(email);
                }
            }
        }
    }

    // Logout
    imap_session
        .logout()
        .map_err(|e| format!("Failed to logout: {}", e))?;

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
