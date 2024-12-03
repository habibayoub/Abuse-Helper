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

/// Represents an outgoing email message.
///
/// Used for composing and sending new emails through SMTP.
///
/// # Fields
/// * `recipient` - Target email address with optional display name
/// * `subject` - Email subject line
/// * `body` - Plain text email content
#[derive(Debug, Deserialize, Clone)]
pub struct OutgoingEmail {
    pub recipient: Mailbox,
    pub subject: String,
    pub body: String,
}

/// Comprehensive email record structure.
///
/// Represents both incoming and outgoing emails with full metadata.
///
/// # Fields
/// * `id` - Unique identifier
/// * `sender` - Email sender address
/// * `recipients` - List of recipient addresses
/// * `subject` - Email subject
/// * `body` - Email content
/// * `received_at` - Reception timestamp
/// * `analyzed` - Threat analysis status
/// * `is_sent` - Outgoing email indicator
/// * `ticket_ids` - Associated security tickets
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

/// Comprehensive error type for email operations.
///
/// Handles all potential failure modes in email processing pipeline.
///
/// # Categories
/// - Database errors
/// - SMTP/IMAP errors
/// - Threat analysis failures
/// - Configuration issues
/// - Validation problems
/// - Connection pool errors
/// - Elasticsearch integration errors
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

/// Search filter criteria for email queries.
///
/// Enables flexible email searching with multiple filter options.
///
/// # Fields
/// * `is_sent` - Filter by outgoing status
/// * `analyzed` - Filter by analysis status
/// * `has_tickets` - Filter by ticket association
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilters {
    pub is_sent: Option<bool>,
    pub analyzed: Option<bool>,
    pub has_tickets: Option<bool>,
}

/// Search configuration for email queries.
///
/// Combines search text with filtering and pagination options.
///
/// # Fields
/// * `query` - Search text
/// * `filters` - Optional filter criteria
/// * `from` - Pagination offset
/// * `size` - Results per page
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub from: Option<usize>,
    pub size: Option<usize>,
}

/// Search results container.
///
/// Wraps matching emails with pagination metadata.
///
/// # Fields
/// * `hits` - Matching email records
/// * `total` - Total result count
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<Email>,
    pub total: u64,
}

impl Email {
    /// Marks an email as analyzed after threat assessment.
    ///
    /// Updates both database and Elasticsearch records.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<(), EmailError>` - Success or specific error
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

    /// Retrieves new emails from IMAP server.
    ///
    /// Connects to configured IMAP server and fetches unread messages.
    ///
    /// # Environment Variables
    /// * `IMAP_SERVER` - Server hostname
    /// * `IMAP_PORT` - Server port
    ///
    /// # Returns
    /// * `Result<Vec<Email>, EmailError>` - New emails or error
    async fn fetch_from_imap() -> Result<Vec<Email>, EmailError> {
        // Get the IMAP server and port from environment variables
        let imap_server = std::env::var("IMAP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
        let imap_port = std::env::var("IMAP_PORT")
            .unwrap_or_else(|_| "3993".to_string())
            .parse::<u16>()
            .unwrap_or(3993);

        log::info!("Connecting to IMAP server: {}", imap_server);

        // Create a TLS connector with disabled certificate validation
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| {
                EmailError::Validation(format!("Failed to create TLS connector: {}", e))
            })?;

        // Connect to the IMAP server
        let client = imap::connect((imap_server.clone(), imap_port), imap_server, &tls)
            .map_err(|e| EmailError::Validation(format!("Failed to connect to IMAP: {}", e)))?;

        // Login to the IMAP server
        let mut imap_session = client
            .login("test@localhost", "password")
            .map_err(|e| EmailError::Validation(format!("Failed to login to IMAP: {:?}", e)))?;

        let mut emails = Vec::new();

        // Select the INBOX mailbox
        let mailbox = imap_session
            .select("INBOX")
            .map_err(|e| EmailError::Validation(format!("Failed to select INBOX: {}", e)))?;

        // Check if there are any messages in the INBOX
        if mailbox.exists > 0 {
            log::info!("INBOX has {} messages", mailbox.exists);

            // Fetch messages from the INBOX
            let sequence_set = format!("1:{}", mailbox.exists);
            let messages = imap_session
                .fetch(sequence_set, "RFC822")
                .map_err(|e| EmailError::Validation(format!("Failed to fetch messages: {}", e)))?;

            // Process each message
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

        // Logout from the IMAP server
        imap_session
            .logout()
            .map_err(|e| EmailError::Validation(format!("Failed to logout: {}", e)))?;

        Ok(emails)
    }

    /// Fetch emails from database
    async fn fetch_from_db(pool: &Pool) -> Result<Vec<Email>, EmailError> {
        // Log the start of the database fetch operation
        log::info!("Fetching emails from database");

        // Get a connection from the pool
        let client = pool.get().await?;

        // Execute the query to fetch emails from the database
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

        // Convert the rows to a vector of Email structs
        let emails: Vec<Email> = rows
            .iter()
            .map(|row| {
                let mut email = Email::from(row.clone());
                email.ticket_ids = row.get("ticket_ids");
                email
            })
            .collect();

        // Log the number of emails found in the database
        log::info!("Found {} emails in database", emails.len());

        // Return the vector of emails
        Ok(emails)
    }

    /// Fetch all emails from both database and IMAP
    pub async fn fetch_all(pool: &Pool) -> Result<Vec<Email>, EmailError> {
        // Log the start of the fetch operation
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
            // Check if the email already exists in the database
            if !existing_ids.contains(&email.id) {
                // Save the new email to the database
                if let Err(e) = email.save(pool).await {
                    log::error!("Failed to save email: {}", e);
                } else {
                    // Add the new email to the existing IDs set
                    existing_ids.insert(email.id.clone());
                    // Add the new email to the emails vector
                    emails.push(email);
                    // Increment the new count
                    new_count += 1;
                }
            }
        }

        // Log the number of new emails added
        log::info!("Added {} new emails", new_count);

        // Return the combined vector of emails
        Ok(emails)
    }

    /// Create a ticket from this email
    pub async fn create_ticket(&self, pool: &Pool) -> Result<Uuid, EmailError> {
        // Log the start of the ticket creation process
        log::info!("Creating ticket for email {}", self.id);

        // Analyze the email content
        let analysis = analyze_threat(&self.content()).await?;

        // Extract the IP address from the indicators
        let ip_address = analysis
            .extracted_indicators
            .iter()
            .find(|indicator| indicator.contains('.'))
            .cloned();

        // Create an enhanced description for the ticket
        let enhanced_description = format!(
            "Original Content:\n{}\n\nThreat Analysis:\n- Confidence: {}\n- Identified Threats: {}\n- Extracted Indicators: {}\n\nSummary: {}",
            self.body,
            analysis.confidence_score,
            analysis.identified_threats.join(", "),
            analysis.extracted_indicators.join(", "),
            analysis.summary
        );

        // Create a new ticket
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

        // Get a connection from the pool
        let mut client = pool.get().await?;
        // Start a transaction
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

        // Log the successful creation of the ticket
        log::info!(
            "Created ticket {} and linked it to email {}",
            ticket_id,
            self.id
        );

        // Return the ticket ID
        Ok(ticket_id)
    }

    /// Get associated tickets for this email
    pub async fn get_tickets(&self, pool: &Pool) -> Result<Vec<Uuid>, EmailError> {
        // Get a connection from the pool
        let client = pool.get().await?;

        // Execute the query to fetch ticket IDs associated with this email
        let rows = client
            .query(
                "SELECT ticket_id FROM email_tickets WHERE email_id = $1",
                &[&self.id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        // Return the ticket IDs
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

        // If the ticket does not exist, return an error
        if !ticket_exists {
            return Err(EmailError::Validation(format!(
                "Ticket {} does not exist",
                ticket_id
            )));
        }

        // Insert the link into the database
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
        // Get a connection from the pool
        let client = pool.get().await?;

        // Delete the link from the database
        let result = client
            .execute(
                "DELETE FROM email_tickets WHERE email_id = $1 AND ticket_id = $2",
                &[&self.id, &ticket_id],
            )
            .await
            .map_err(|e| EmailError::Database(e))?;

        // If the link was not found, return an error
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
        // Create a vector of futures for each email
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

        // Wait for all futures to complete
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
        // Get a connection from the pool
        let client = pool.get().await?;

        // Insert the email into the database
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
        // Log the start of the batch processing
        log::info!("Processing batch of {} emails", ids.len());

        // Get a connection from the pool
        let client = pool.get().await?;

        // Create a vector to store the results
        let mut results = Vec::with_capacity(ids.len());

        // Process each email
        for id in ids {
            // Fetch the email from the database
            let row = client
                .query_opt("SELECT * FROM emails WHERE id = $1", &[&id])
                .await?;

            if let Some(row) = row {
                // Convert the row to an email
                let mut email = Email::from(row);

                // Create a ticket for the email
                match email.create_ticket(pool).await {
                    Ok(_) => {
                        // Mark the email as analyzed
                        if let Err(e) = email.mark_as_analyzed(pool).await {
                            log::error!("Failed to mark email {} as analyzed: {}", id, e);
                            results.push(Err(e));
                            continue;
                        }
                        // Add the result to the results vector
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

        // If the email has any associated tickets, return an error
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
        // Get a connection from the pool
        let client = pool.get().await?;

        // Fetch the email from the database
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
                // Convert the row to an email
                let mut email = Email::from(row.clone());
                email.ticket_ids = row.get("ticket_ids");
                Ok(email)
            }
            None => Err(EmailError::Validation(format!("Email {} not found", id))),
        }
    }

    /// Index this email to ElasticSearch
    pub async fn index_to_es(&self) -> Result<(), ESError> {
        // Get a connection to ElasticSearch
        let client = ESClient::new().await?;

        // Create the document to index
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
        // Get a connection to ElasticSearch
        let client = ESClient::new().await?;

        // Create the query
        let mut query = json!({
            "bool": {
                "must": [{
                    "multi_match": {
                        "query": options.query,
                        "fields": ["subject^2", "body", "sender", "recipients"],
                        "fuzziness": "AUTO"
                    }
                }],
                "filter": []
            }
        });

        // Add filters if provided
        if let Some(filters) = options.filters {
            // Add the is_sent filter if provided
            if let Some(is_sent) = filters.is_sent {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term": {"is_sent": is_sent}}));
            }

            // Add the has_tickets filter if provided
            if let Some(has_tickets) = filters.has_tickets {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(if has_tickets {
                        json!({
                            "exists": {
                                "field": "ticket_ids"
                            }
                        })
                    } else {
                        json!({
                            "bool": {
                                "must_not": {
                                    "exists": {
                                        "field": "ticket_ids"
                                    }
                                }
                            }
                        })
                    });
            }
        }

        // Create the search body
        let search_body = json!({
            "query": query,
            "sort": [{ "received_at": { "order": "desc" } }],
            "from": options.from.unwrap_or(0),
            "size": options.size.unwrap_or(50)
        });

        // Perform the search
        let result = client.search::<Email>("emails", search_body).await?;

        // Return the search response
        Ok(SearchResponse {
            hits: result.hits,
            total: result.total,
        })
    }
}

impl OutgoingEmail {
    /// Save sent email to database
    pub async fn save(&self, pool: &Pool) -> Result<String, EmailError> {
        // Log the start of the save operation
        log::info!("Saving sent email to {}", self.recipient.email);

        // Get a connection from the pool
        let client = pool.get().await?;

        // Create a new UUID for the email
        let id = Uuid::new_v4().to_string();

        // Get the SMTP username from the environment variables
        let smtp_username =
            env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

        // Prepare the SQL statement
        let stmt = client
            .prepare(
                "INSERT INTO emails (id, sender, recipients, subject, body, received_at, is_sent, analyzed) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
                 RETURNING id",
            )
            .await
            .map_err(|e| EmailError::Database(e.into()))?;

        // Execute the SQL statement
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

        // Return the ID of the saved email
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

    /// Sends email via configured SMTP server.
    ///
    /// # Environment Variables
    /// * `SMTP_SERVER` - Server hostname
    /// * `SMTP_PORT` - Server port
    /// * `SMTP_USERNAME` - Sender address
    ///
    /// # Returns
    /// * `Result<String, EmailError>` - Success message or error
    pub async fn send(&self) -> Result<String, EmailError> {
        // Log the start of the send operation
        log::info!("Sending email to {}", self.recipient.email);

        // Get the SMTP server from the environment variables
        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "mailserver".to_string());
        let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "3025".to_string());
        let smtp_username =
            env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@localhost".to_string());

        // Parse the recipient address
        let recipient = self.recipient.email.clone();
        let from_address = smtp_username
            .parse::<Mailbox>()
            .map_err(|e| EmailError::Validation(e.to_string()))?;

        // Build the email payload
        let email_payload = Message::builder()
            .from(from_address)
            .to(self.recipient.clone())
            .subject(&self.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(self.body.clone())
            .map_err(|e| EmailError::Validation(e.to_string()))?;

        // Build the SMTP transport
        let mailer =
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_server)
                .port(smtp_port.parse::<u16>().unwrap_or(3025))
                .build();

        // Send the email
        mailer
            .send(email_payload)
            .await
            .map(|_| format!("Successfully sent {} to {}", self.subject, recipient))
            .map_err(|e| EmailError::Smtp(e))
    }
}

/// Database row conversion implementation.
///
/// Maps database columns to Email struct fields.
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

/// String error conversion implementation.
///
/// Converts string errors to EmailError::ThreatAnalysis variant.
impl From<String> for EmailError {
    fn from(error: String) -> Self {
        EmailError::ThreatAnalysis(error)
    }
}

/// Pool error conversion implementation.
///
/// Converts connection pool errors to EmailError::Pool variant.
impl From<deadpool_postgres::PoolError> for EmailError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        EmailError::Pool(error.to_string())
    }
}
