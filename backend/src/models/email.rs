use crate::llm::analyze_threat;
use crate::models::es::{ESClient, ESError};
use crate::models::ticket::Ticket;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use futures::future;
use lettre::{message::header::ContentType, message::Mailbox, AsyncTransport, Message};
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    pub id: Uuid,
    pub sender: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub received_at: DateTime<Utc>,
    pub analyzed: bool,
    pub is_sent: bool,
    #[serde(default)]
    pub ticket_ids: Vec<Uuid>,
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

    /// ElasticSearch errors
    #[error("ElasticSearch error: {0}")]
    ES(#[from] ESError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilters {
    pub is_sent: Option<bool>,
    pub analyzed: Option<bool>,
    pub has_tickets: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub from: Option<usize>,
    pub size: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<Email>,
    pub total: u64,
    pub suggestions: Vec<String>,
}

impl Email {
    /// Mark email as analyzed
    pub async fn mark_as_analyzed(&mut self, pool: &Pool) -> Result<(), EmailError> {
        log::info!("Marking email {} as analyzed", self.id);
        let client = pool.get().await?;

        // Update in database
        client
            .execute(
                "UPDATE emails SET analyzed = TRUE WHERE id = $1",
                &[&self.id],
            )
            .await
            .map_err(|e| EmailError::Pool(e.to_string()))?;

        // Update in ElasticSearch
        let es_client = ESClient::new().await?;
        if let Err(e) = es_client
            .update_document(
                "emails",
                &self.id.to_string(),
                &json!({
                    "analyzed": true
                }),
            )
            .await
        {
            log::error!("Failed to update email in ElasticSearch: {}", e);
        }

        self.analyzed = true;
        Ok(())
    }

    /// Fetch emails from IMAP server
    async fn fetch_from_imap() -> Result<Vec<Email>, EmailError> {
        let imap_server = std::env::var("IMAP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
        let imap_port = std::env::var("IMAP_PORT")
            .unwrap_or_else(|_| "3993".to_string())
            .parse::<u16>()
            .unwrap_or(3993);

        log::info!("Connecting to IMAP server: {}", imap_server);

        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| {
                EmailError::Validation(format!("Failed to create TLS connector: {}", e))
            })?;

        let client = imap::connect((imap_server.clone(), imap_port), imap_server, &tls)
            .map_err(|e| EmailError::Validation(format!("Failed to connect to IMAP: {}", e)))?;

        let mut imap_session = client
            .login("test@localhost", "password")
            .map_err(|e| EmailError::Validation(format!("Failed to login to IMAP: {:?}", e)))?;

        let mut emails = Vec::new();

        let mailbox = imap_session
            .select("INBOX")
            .map_err(|e| EmailError::Validation(format!("Failed to select INBOX: {}", e)))?;

        if mailbox.exists > 0 {
            log::info!("INBOX has {} messages", mailbox.exists);

            let sequence_set = format!("1:{}", mailbox.exists);
            let messages = imap_session
                .fetch(sequence_set, "RFC822")
                .map_err(|e| EmailError::Validation(format!("Failed to fetch messages: {}", e)))?;

            for message in messages.iter() {
                if let Some(body) = message.body() {
                    if let Ok(parsed_mail) = parse_mail(body) {
                        let headers = parsed_mail.get_headers();
                        let from = headers.get_first_value("From").unwrap_or_default();
                        let to = headers.get_all_values("To");
                        let subject = headers.get_first_value("Subject").unwrap_or_default();
                        let body = parsed_mail.get_body().unwrap_or_default();

                        emails.push(Email {
                            id: Uuid::new_v4(),
                            sender: from,
                            recipients: to,
                            subject,
                            body,
                            received_at: Utc::now(),
                            analyzed: false,
                            is_sent: false,
                            ticket_ids: Vec::new(),
                        });
                    }
                }
            }
        }

        imap_session
            .logout()
            .map_err(|e| EmailError::Validation(format!("Failed to logout: {}", e)))?;

        Ok(emails)
    }

    /// Fetch emails from database
    async fn fetch_from_db(pool: &Pool) -> Result<Vec<Email>, EmailError> {
        log::info!("Fetching emails from database");
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT e.*, COALESCE(array_agg(et.ticket_id) FILTER (WHERE et.ticket_id IS NOT NULL), ARRAY[]::uuid[]) as ticket_ids 
                 FROM emails e 
                 LEFT JOIN email_tickets et ON e.id = et.email_id 
                 GROUP BY e.id 
                 ORDER BY e.received_at DESC",
                &[],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        let emails: Vec<Email> = rows
            .iter()
            .map(|row| {
                let mut email = Email::from(row.clone());
                email.ticket_ids = row.get("ticket_ids");
                email
            })
            .collect();

        log::info!("Found {} emails in database", emails.len());
        Ok(emails)
    }

    /// Fetch all emails from both database and IMAP
    pub async fn fetch_all(pool: &Pool) -> Result<Vec<Email>, EmailError> {
        log::info!("Fetching all emails from database and IMAP");

        // Fetch from database
        let mut emails = Self::fetch_from_db(pool).await?;
        let mut existing_ids: std::collections::HashSet<Uuid> =
            emails.iter().map(|e| e.id.clone()).collect();

        // Fetch new emails from IMAP
        let imap_emails = Self::fetch_from_imap().await?;
        log::info!("Found {} emails from IMAP", imap_emails.len());

        // Save new emails to database
        let mut new_count = 0;
        for email in imap_emails {
            if !existing_ids.contains(&email.id) {
                if let Err(e) = email.save(pool).await {
                    log::error!("Failed to save email: {}", e);
                } else {
                    existing_ids.insert(email.id.clone());
                    emails.push(email);
                    new_count += 1;
                }
            }
        }

        log::info!("Added {} new emails", new_count);
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

        let enhanced_description = format!(
            "Original Content:\n{}\n\nThreat Analysis:\n- Confidence: {}\n- Identified Threats: {}\n- Extracted Indicators: {}\n\nSummary: {}",
            self.body,
            analysis.confidence_score,
            analysis.identified_threats.join(", "),
            analysis.extracted_indicators.join(", "),
            analysis.summary
        );

        let ticket = Ticket::new(
            analysis.threat_type,
            self.subject.clone(),
            enhanced_description,
            ip_address,
            Some(analysis.confidence_score as f64),
            Some(analysis.identified_threats),
            Some(analysis.extracted_indicators),
            Some(analysis.summary),
        );

        let mut client = pool.get().await?;
        let tx = client.transaction().await?;

        // Save the ticket within the transaction
        let ticket_id = match ticket.save_with_client(&tx).await {
            Ok(id) => id,
            Err(e) => {
                let _ = tx.rollback().await;
                log::error!("Failed to save ticket: {}", e);
                return Err(EmailError::ThreatAnalysis(e.to_string()));
            }
        };

        // Link this email to the ticket
        if let Err(e) = ticket.add_email_with_client(&tx, &self.id).await {
            let _ = tx.rollback().await;
            log::error!("Failed to link email: {}", e);
            return Err(EmailError::ThreatAnalysis(e.to_string()));
        }

        // Commit the transaction
        if let Err(e) = tx.commit().await {
            log::error!("Failed to commit transaction: {}", e);
            return Err(EmailError::Database(e));
        }

        log::info!(
            "Created ticket {} and linked it to email {}",
            ticket_id,
            self.id
        );
        Ok(ticket_id)
    }

    /// Get associated tickets for this email
    pub async fn get_tickets(&self, pool: &Pool) -> Result<Vec<Uuid>, EmailError> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT ticket_id FROM email_tickets WHERE email_id = $1",
                &[&self.id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        Ok(rows.iter().map(|row| row.get("ticket_id")).collect())
    }

    /// Link this email to a ticket
    pub async fn link_ticket(&self, pool: &Pool, ticket_id: Uuid) -> Result<(), EmailError> {
        let client = pool.get().await?;

        // First verify the ticket exists
        let ticket_exists = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM tickets WHERE id = $1)",
                &[&ticket_id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?
            .get::<_, bool>(0);

        if !ticket_exists {
            return Err(EmailError::Validation(format!(
                "Ticket {} does not exist",
                ticket_id
            )));
        }

        client
            .execute(
                "INSERT INTO email_tickets (email_id, ticket_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&self.id, &ticket_id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        Ok(())
    }

    /// Unlink this email from a ticket
    pub async fn unlink_ticket(&self, pool: &Pool, ticket_id: Uuid) -> Result<(), EmailError> {
        let client = pool.get().await?;

        let result = client
            .execute(
                "DELETE FROM email_tickets WHERE email_id = $1 AND ticket_id = $2",
                &[&self.id, &ticket_id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        if result == 0 {
            return Err(EmailError::Validation(format!(
                "Email {} is not linked to ticket {}",
                self.id, ticket_id
            )));
        }

        Ok(())
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

    /// Create a new email instance
    pub fn new(sender: String, recipients: Vec<String>, subject: String, body: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipients,
            subject,
            body,
            received_at: Utc::now(),
            analyzed: false,
            is_sent: false,
            ticket_ids: Vec::new(),
        }
    }

    /// Save email to database
    pub async fn save(&self, pool: &Pool) -> Result<(), EmailError> {
        let client = pool.get().await?;

        client
            .execute(
                "INSERT INTO emails (id, sender, recipients, subject, body, received_at, analyzed, is_sent) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &self.id,
                    &self.sender,
                    &self.recipients,
                    &self.subject,
                    &self.body,
                    &self.received_at,
                    &self.analyzed,
                    &self.is_sent,
                ],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        // Index to ElasticSearch
        if let Err(e) = self.index_to_es().await {
            log::error!("Failed to index email to ElasticSearch: {}", e);
        }

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

    /// Process multiple emails by their IDs
    pub async fn process_batch_by_ids(
        pool: &Pool,
        ids: &[Uuid],
    ) -> Result<Vec<Result<(), EmailError>>, EmailError> {
        log::info!("Processing batch of {} emails", ids.len());
        let client = pool.get().await?;

        let mut results = Vec::with_capacity(ids.len());

        for id in ids {
            let row = client
                .query_opt("SELECT * FROM emails WHERE id = $1", &[&id])
                .await?;

            if let Some(row) = row {
                let mut email = Email::from(row);

                match email.create_ticket(pool).await {
                    Ok(_) => {
                        if let Err(e) = email.mark_as_analyzed(pool).await {
                            log::error!("Failed to mark email {} as analyzed: {}", id, e);
                            results.push(Err(e));
                            continue;
                        }
                        results.push(Ok(()));
                    }
                    Err(e) => {
                        log::error!("Failed to create ticket for email {}: {}", id, e);
                        results.push(Err(e));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Safely delete this email, checking for ticket associations first
    pub async fn delete(&self, pool: &Pool) -> Result<(), EmailError> {
        let client = pool.get().await?;

        // Check if email has any associated tickets
        let has_tickets = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM email_tickets WHERE email_id = $1)",
                &[&self.id],
            )
            .await?
            .get::<_, bool>(0);

        if has_tickets {
            return Err(EmailError::Validation(format!(
                "Cannot delete email {} because it is associated with one or more tickets",
                self.id
            )));
        }

        // If no tickets, proceed with deletion
        client
            .execute("DELETE FROM emails WHERE id = $1", &[&self.id])
            .await
            .map_err(|e| EmailError::Database(e.into()))?;

        Ok(())
    }

    /// Force delete this email and remove all ticket associations
    pub async fn force_delete(&self, pool: &Pool) -> Result<(), EmailError> {
        let mut client = pool.get().await?;
        let tx = client.transaction().await?;

        // Remove from database
        if let Err(e) = tx
            .execute("DELETE FROM email_tickets WHERE email_id = $1", &[&self.id])
            .await
        {
            let _ = tx.rollback().await;
            return Err(EmailError::Database(e));
        }

        if let Err(e) = tx
            .execute("DELETE FROM emails WHERE id = $1", &[&self.id])
            .await
        {
            let _ = tx.rollback().await;
            return Err(EmailError::Database(e));
        }

        // Remove from ElasticSearch
        let es_client = ESClient::new().await?;
        if let Err(e) = es_client
            .delete_document("emails", &self.id.to_string())
            .await
        {
            let _ = tx.rollback().await;
            return Err(EmailError::ES(e));
        }

        tx.commit().await?;
        Ok(())
    }

    /// Fetch a single email by ID
    pub async fn fetch_by_id(pool: &Pool, id: &Uuid) -> Result<Email, EmailError> {
        let client = pool.get().await?;

        let row = client
            .query_opt(
                "SELECT e.*, COALESCE(array_agg(et.ticket_id) FILTER (WHERE et.ticket_id IS NOT NULL), ARRAY[]::uuid[]) as ticket_ids 
                 FROM emails e 
                 LEFT JOIN email_tickets et ON e.id = et.email_id 
                 WHERE e.id = $1 
                 GROUP BY e.id",
                &[&id],
            )
            .await?;

        match row {
            Some(row) => {
                let mut email = Email::from(row.clone());
                email.ticket_ids = row.get("ticket_ids");
                Ok(email)
            }
            None => Err(EmailError::Validation(format!("Email {} not found", id))),
        }
    }

    /// Index this email to ElasticSearch
    pub async fn index_to_es(&self) -> Result<(), ESError> {
        let client = ESClient::new().await?;

        let document = json!({
            "id": self.id,
            "sender": self.sender,
            "recipients": self.recipients,
            "subject": self.subject,
            "body": self.body,
            "received_at": self.received_at,
            "analyzed": self.analyzed,
            "is_sent": self.is_sent,
            "ticket_ids": self.ticket_ids
        });

        client
            .index_document("emails", &self.id.to_string(), &document)
            .await
    }

    pub async fn search(options: SearchOptions) -> Result<SearchResponse, EmailError> {
        let client = ESClient::new().await?;

        let mut query = json!({
            "query": {
                "multi_match": {
                    "query": options.query,
                    "fields": ["subject^2", "body", "sender", "recipients"],
                    "fuzziness": "AUTO"
                }
            },
            "sort": [{ "received_at": { "order": "desc" } }],
            "from": options.from.unwrap_or(0),
            "size": options.size.unwrap_or(50)
        });

        // Add filters if provided
        if let Some(filters) = options.filters {
            log::debug!("Applying filters: {:?}", filters);

            if let Some(is_sent) = filters.is_sent {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term": {"is_sent": is_sent}}));
            }

            if let Some(has_tickets) = filters.has_tickets {
                query["bool"]["filter"].as_array_mut().unwrap().push(json!({
                    "script": {
                        "script": {
                            "source": if has_tickets {
                                "doc['ticket_ids'].length > 0"
                            } else {
                                "doc['ticket_ids'].length == 0"
                            }
                        }
                    }
                }));
            }
        }

        let result = client.search::<Email>("emails", query).await?;

        Ok(SearchResponse {
            hits: result.hits,
            total: result.total,
            suggestions: vec![],
        })
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
                "INSERT INTO emails (id, sender, recipients, subject, body, received_at, is_sent, analyzed) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
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
                    &true,
                    &false,
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
            is_sent: row.get("is_sent"),
            ticket_ids: Vec::new(),
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
