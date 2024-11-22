use crate::llm::analyze_threat;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use futures::future;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use serde::{Deserialize, Serialize};
use std::env;
use tokio_postgres::Row;
use uuid::Uuid;

/// Struct representing an outgoing email
#[derive(Debug, Deserialize, Clone)]
pub struct OutgoingEmail {
    pub recipient: Mailbox,
    pub subject: String,
    pub body: String,
}

/// Struct representing a received or stored email
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Email {
    pub id: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub received_at: DateTime<Utc>,
    pub analyzed: bool,
}

/// Represents all possible errors that can occur when handling emails
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    /// SMTP/email sending errors
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),

    /// Errors from threat analysis
    #[error("Failed to analyze threat: {0}")]
    ThreatAnalysis(String),

    /// Environment variable errors
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Connection pool errors
    #[error("Pool error: {0}")]
    Pool(String),
}

impl Email {
    /// Mark email as analyzed
    pub async fn mark_as_analyzed(&mut self, pool: &Pool) -> Result<(), EmailError> {
        log::info!("Marking email {} as analyzed", self.id);
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE emails SET analyzed = TRUE WHERE id = $1",
                &[&self.id],
            )
            .await
            .map_err(|e| EmailError::Pool(e.to_string()))?;

        self.analyzed = true;
        Ok(())
    }

    /// Fetch all unanalyzed emails
    pub async fn fetch_unanalyzed(pool: &Pool) -> Result<Vec<Email>, EmailError> {
        log::info!("Fetching unanalyzed emails");
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT * FROM emails WHERE analyzed = FALSE ORDER BY received_at DESC",
                &[],
            )
            .await
            .map_err(|e| EmailError::Database(e.into()))?;

        let emails: Vec<Email> = rows
            .into_iter()
            .map(Email::from)
            .filter(|email| !email.is_analyzed())
            .collect();

        log::info!("Found {} unanalyzed emails", emails.len());
        Ok(emails)
    }

    /// Create a ticket from this email
    pub async fn create_ticket(&self, pool: &Pool) -> Result<Uuid, EmailError> {
        log::info!("Creating ticket for email {}", self.id);
        let analysis = analyze_threat(&self.content()).await?;

        let ip_address = analysis
            .extracted_indicators
            .iter()
            .find(|indicator| indicator.contains('.'))
            .cloned();

        let ticket_id = Uuid::new_v4();
        let client = pool.get().await?;

        let enhanced_description = format!(
            "Original Content:\n{}\n\nThreat Analysis:\n- Confidence: {}\n- Identified Threats: {}\n- Extracted Indicators: {}\n\nSummary: {}",
            self.body,
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
            .map_err(|e| EmailError::Database(e.into()))?;

        client
            .query_one(
                &stmt,
                &[
                    &ticket_id,
                    &analysis.threat_type.to_string(),
                    &ip_address,
                    &self.id,
                    &self.subject,
                    &enhanced_description,
                    &analysis.confidence_score,
                    &analysis.identified_threats,
                    &analysis.extracted_indicators,
                    &analysis.summary,
                ],
            )
            .await
            .map_err(|e| EmailError::Pool(e.to_string()))?;

        Ok(ticket_id)
    }

    /// Process multiple emails in parallel
    pub async fn process_batch(pool: &Pool, emails: &mut [Email]) -> Vec<Result<(), EmailError>> {
        let futures: Vec<_> = emails
            .iter_mut()
            .map(|email| async move {
                if let Err(e) = email.create_ticket(pool).await {
                    log::error!("Failed to create ticket for email {}: {}", email.id, e);
                    return Err(e);
                }

                email.mark_as_analyzed(pool).await?;
                Ok(())
            })
            .collect();

        future::join_all(futures).await
    }

    #[allow(dead_code)]
    /// Create a new email instance
    pub fn new(sender: String, recipients: Vec<String>, subject: String, body: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            recipients,
            subject,
            body,
            received_at: Utc::now(),
            analyzed: false,
        }
    }

    #[allow(dead_code)]
    /// Save email to database
    pub async fn save(&self, pool: &Pool) -> Result<(), EmailError> {
        log::info!("Saving email from {}", self.sender);
        let client = pool.get().await?;

        let stmt = client
            .prepare(
                "INSERT INTO emails (id, sender, recipients, subject, body, received_at, analyzed) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .await
            .map_err(|e| EmailError::Database(e.into()))?;

        client
            .execute(
                &stmt,
                &[
                    &self.id,
                    &self.sender,
                    &self.recipients,
                    &self.subject,
                    &self.body,
                    &self.received_at,
                    &self.analyzed,
                ],
            )
            .await
            .map_err(|e| EmailError::Pool(e.to_string()))?;

        Ok(())
    }

    /// Check if the email has been analyzed
    pub fn is_analyzed(&self) -> bool {
        self.analyzed
    }

    /// Get the formatted email content
    pub fn content(&self) -> String {
        format!(
            "From: {}\nTo: {}\nSubject: {}\nBody: {}",
            self.sender,
            self.recipients.join(", "),
            self.subject,
            self.body
        )
    }
}

impl OutgoingEmail {
    /// Save sent email to database
    pub async fn save(&self, pool: &Pool) -> Result<String, EmailError> {
        log::info!("Saving sent email to {}", self.recipient.email);
        let client = pool.get().await?;
        let id = Uuid::new_v4().to_string();
        let smtp_username =
            env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

        let stmt = client
            .prepare(
                "INSERT INTO emails (id, sender, recipients, subject, body, received_at) 
                 VALUES ($1, $2, $3, $4, $5, $6) 
                 RETURNING id",
            )
            .await
            .map_err(|e| EmailError::Database(e.into()))?;

        client
            .query_one(
                &stmt,
                &[
                    &id,
                    &smtp_username,
                    &vec![self.recipient.email.to_string()],
                    &self.subject,
                    &self.body,
                    &Utc::now(),
                ],
            )
            .await
            .map_err(|e| EmailError::Pool(e.to_string()))?;

        Ok(id)
    }

    pub fn validate(&self) -> Result<(), EmailError> {
        if self.subject.is_empty() {
            return Err(EmailError::Validation("Subject cannot be empty".into()));
        }
        if self.body.is_empty() {
            return Err(EmailError::Validation("Body cannot be empty".into()));
        }
        Ok(())
    }

    /// Send an email using SMTP
    pub async fn send(&self) -> Result<String, EmailError> {
        log::info!("Sending email to {}", self.recipient.email);
        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
        let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "3025".to_string());
        let smtp_username =
            env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

        let recipient = self.recipient.email.clone();
        let from_address = smtp_username
            .parse::<Mailbox>()
            .map_err(|e| EmailError::Validation(e.to_string()))?;

        let email_payload = Message::builder()
            .from(from_address)
            .to(self.recipient.clone())
            .subject(&self.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(self.body.clone())
            .map_err(|e| EmailError::Validation(e.to_string()))?;

        let mailer =
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_server)
                .port(smtp_port.parse::<u16>().unwrap_or(3025))
                .build();

        mailer
            .send(email_payload)
            .await
            .map(|_| format!("Successfully sent {} to {}", self.subject, recipient))
            .map_err(|e| EmailError::Smtp(e))
    }
}

impl From<Row> for Email {
    fn from(row: Row) -> Self {
        Email {
            id: row.get("id"),
            sender: row.get("sender"),
            recipients: row.get("recipients"),
            subject: row.get("subject"),
            body: row.get("body"),
            received_at: row.get("received_at"),
            analyzed: row.get("analyzed"),
        }
    }
}

impl From<String> for EmailError {
    fn from(error: String) -> Self {
        EmailError::ThreatAnalysis(error)
    }
}

impl From<deadpool_postgres::PoolError> for EmailError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        EmailError::Pool(error.to_string())
    }
}
