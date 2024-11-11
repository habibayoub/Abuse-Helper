use crate::{
    llm::analyze_threat,
    models::email::{Email, OutgoingEmail},
};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use reqwest;
use serde_json::Value;
use std::env;
use uuid::Uuid;

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
    let mailcrab_url =
        env::var("MAILCRAB_URL").unwrap_or_else(|_| "http://mailcrab:1080".to_string());

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

    log::debug!("Fetched messages: {:#?}", messages); // Debug log to see raw response

    let mut emails = Vec::new();

    for message in messages {
        let email_id = message["id"].as_str().unwrap_or("").to_string();

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

        // Get the email content
        let content_response = client
            .get(format!("{}/api/messages/{}/plain", mailcrab_url, email_id))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch email content: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to get email text: {}", e))?;

        let email = Email {
            id: email_id,
            sender: message["from"]["email"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            recipients: vec![message["to"][0]["email"].as_str().unwrap_or("").to_string()],
            subject: message["subject"]
                .as_str()
                .unwrap_or("No Subject")
                .to_string(),
            body: content_response, // Use the content from the separate API call
            received_at: DateTime::parse_from_rfc3339(
                message["date"].as_str().unwrap_or(&Utc::now().to_rfc3339()),
            )
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc),
            analyzed: false,
        };

        log::debug!("Parsed email: {:#?}", email); // Debug log to see parsed email

        if let Err(e) = create_ticket_from_email(pool, &email).await {
            log::error!("Failed to create ticket for email {}: {}", email.id, e);
        } else {
            log::info!("Created ticket for email: {}", email.id);
        }

        emails.push(email.clone());
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
                log::info!("Found {} new emails", emails.len());
                for email in emails {
                    log::info!(
                        "Email from: {}, subject: {}, received at: {}",
                        email.sender,
                        email.subject,
                        email.received_at
                    );
                }
            }
        }
        Err(e) => log::error!("Failed to poll emails: {}", e),
    }
}
