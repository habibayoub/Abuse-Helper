use crate::models::es::{ESClient, ESError};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_postgres::Row;
use uuid::Uuid;

/// Types of security incidents that can be reported.
///
/// Comprehensive enumeration of security incident categories
/// for classification and tracking purposes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TicketType {
    /// Malicious software detection
    Malware,
    /// Social engineering attempts
    Phishing,
    /// Fraudulent activities
    Scam,
    /// Unsolicited bulk messages
    Spam,
    /// Distributed Denial of Service attacks
    DDoS,
    /// Botnet activity detection
    Botnet,
    /// Information exposure incidents
    DataBreach,
    /// Personal identity compromise
    IdentityTheft,
    /// Encryption-based extortion
    Ransomware,
    /// Online harassment via technology
    CyberStalking,
    /// IP rights violations
    IntellectualPropertyTheft,
    /// Digital harassment cases
    Harassment,
    /// System access violations
    UnauthorizedAccess,
    /// Rights infringement
    CopyrightViolation,
    /// Password attack attempts
    BruteForce,
    /// Command and Control activity
    C2,
    /// Uncategorized incidents
    Other,
}

impl ToString for TicketType {
    fn to_string(&self) -> String {
        match self {
            TicketType::Malware => "Malware",
            TicketType::Phishing => "Phishing",
            TicketType::Scam => "Scam",
            TicketType::Spam => "Spam",
            TicketType::DDoS => "DDoS",
            TicketType::Botnet => "Botnet",
            TicketType::DataBreach => "DataBreach",
            TicketType::IdentityTheft => "IdentityTheft",
            TicketType::Ransomware => "Ransomware",
            TicketType::CyberStalking => "CyberStalking",
            TicketType::IntellectualPropertyTheft => "IntellectualPropertyTheft",
            TicketType::Harassment => "Harassment",
            TicketType::UnauthorizedAccess => "UnauthorizedAccess",
            TicketType::CopyrightViolation => "CopyrightViolation",
            TicketType::BruteForce => "BruteForce",
            TicketType::C2 => "C2",
            TicketType::Other => "Other",
        }
        .to_string()
    }
}

impl From<String> for TicketType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Malware" => TicketType::Malware,
            "Phishing" => TicketType::Phishing,
            "Scam" => TicketType::Scam,
            "Spam" => TicketType::Spam,
            "DDoS" => TicketType::DDoS,
            "Botnet" => TicketType::Botnet,
            "DataBreach" => TicketType::DataBreach,
            "IdentityTheft" => TicketType::IdentityTheft,
            "Ransomware" => TicketType::Ransomware,
            "CyberStalking" => TicketType::CyberStalking,
            "IntellectualPropertyTheft" => TicketType::IntellectualPropertyTheft,
            "Harassment" => TicketType::Harassment,
            "UnauthorizedAccess" => TicketType::UnauthorizedAccess,
            "CopyrightViolation" => TicketType::CopyrightViolation,
            "BruteForce" => TicketType::BruteForce,
            "C2" => TicketType::C2,
            _ => TicketType::Other,
        }
    }
}

/// Ticket processing status indicators.
///
/// Tracks the lifecycle stage of security incident tickets.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TicketStatus {
    /// Initial state, awaiting processing
    Open,
    /// Currently being investigated
    InProgress,
    /// Investigation completed
    Closed,
    /// Issue successfully addressed
    Resolved,
}

impl ToString for TicketStatus {
    fn to_string(&self) -> String {
        match self {
            TicketStatus::Open => "Open",
            TicketStatus::InProgress => "InProgress",
            TicketStatus::Closed => "Closed",
            TicketStatus::Resolved => "Resolved",
        }
        .to_string()
    }
}

impl From<String> for TicketStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "InProgress" => TicketStatus::InProgress,
            "Closed" => TicketStatus::Closed,
            "Resolved" => TicketStatus::Resolved,
            _ => TicketStatus::Open,
        }
    }
}

impl Default for TicketStatus {
    fn default() -> Self {
        TicketStatus::Open
    }
}

/// Core ticket data structure.
///
/// Represents a security incident ticket with full metadata
/// and relationship tracking.
///
/// # Fields
/// * `id` - Unique identifier
/// * `ticket_type` - Incident classification
/// * `status` - Current processing status
/// * `ip_address` - Related IP address
/// * `subject` - Brief incident description
/// * `description` - Detailed incident information
/// * `confidence_score` - Threat assessment confidence (0.0-1.0)
/// * `identified_threats` - Detected threat indicators
/// * `extracted_indicators` - Security-relevant data points
/// * `analysis_summary` - Threat analysis results
/// * `created_at` - Creation timestamp
/// * `updated_at` - Last modification timestamp
/// * `email_ids` - Associated email identifiers
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    pub ip_address: Option<String>,
    pub subject: String,
    pub description: String,
    pub confidence_score: Option<f64>,
    pub identified_threats: Option<Vec<String>>,
    pub extracted_indicators: Option<Vec<String>>,
    pub analysis_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub email_ids: Vec<Uuid>,
}

impl From<Row> for Ticket {
    fn from(row: Row) -> Self {
        Ticket {
            id: row.get("id"),
            ticket_type: TicketType::from(row.get::<_, String>("ticket_type")),
            status: TicketStatus::from(row.get::<_, String>("status")),
            ip_address: row.get("ip_address"),
            subject: row.get("subject"),
            description: row.get("description"),
            confidence_score: row.get("confidence_score"),
            identified_threats: row.get("identified_threats"),
            extracted_indicators: row.get("extracted_indicators"),
            analysis_summary: row.get("analysis_summary"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            email_ids: Vec::new(),
        }
    }
}

/// Comprehensive error type for ticket operations.
///
/// Handles all potential failure modes in ticket processing pipeline.
///
/// # Categories
/// - Database errors
/// - Connection pool errors
/// - Validation failures
/// - Elasticsearch errors
#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("ElasticSearch error: {0}")]
    ES(#[from] ESError),
}

impl From<deadpool_postgres::PoolError> for TicketError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        TicketError::Pool(error.to_string())
    }
}

/// Search filter criteria for ticket queries.
///
/// Enables flexible ticket searching with multiple filter options.
///
/// # Fields
/// * `status` - Filter by processing status
/// * `ticket_type` - Filter by incident type
/// * `has_emails` - Filter by email association
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilters {
    pub status: Option<TicketStatus>,
    pub ticket_type: Option<TicketType>,
    pub has_emails: Option<bool>,
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
    pub hits: Vec<Ticket>,
    pub total: u64,
}

impl Ticket {
    /// Creates a new ticket instance.
    ///
    /// # Arguments
    /// * `ticket_type` - Incident classification
    /// * `subject` - Brief description
    /// * `description` - Detailed information
    /// * `ip_address` - Related IP
    /// * `confidence_score` - Threat confidence
    /// * `identified_threats` - Detected threats
    /// * `extracted_indicators` - Security indicators
    /// * `analysis_summary` - Analysis results
    pub fn new(
        ticket_type: TicketType,
        subject: String,
        description: String,
        ip_address: Option<String>,
        confidence_score: Option<f64>,
        identified_threats: Option<Vec<String>>,
        extracted_indicators: Option<Vec<String>>,
        analysis_summary: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            ticket_type,
            status: TicketStatus::Open,
            ip_address,
            subject,
            description,
            confidence_score,
            identified_threats,
            extracted_indicators,
            analysis_summary,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            email_ids: Vec::new(),
        }
    }

    /// Persists ticket to database and search index.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Uuid, TicketError>` - Ticket ID or error
    pub async fn save(&self, pool: &Pool) -> Result<Uuid, TicketError> {
        log::info!("Saving ticket {}", self.id);
        let client = pool.get().await?;

        let stmt = client
            .prepare(
                "INSERT INTO tickets (
                    id, ticket_type, status, ip_address, subject, description,
                    confidence_score, identified_threats, extracted_indicators, analysis_summary,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::text[], $9::text[], $10, $11, $12) 
                RETURNING id",
            )
            .await?;

        // Execute the insert statement
        let row = client
            .query_one(
                &stmt,
                &[
                    &self.id,
                    &self.ticket_type.to_string(),
                    &self.status.to_string(),
                    &self.ip_address,
                    &self.subject,
                    &self.description,
                    &self.confidence_score,
                    &self.identified_threats.as_ref().unwrap_or(&vec![]),
                    &self.extracted_indicators.as_ref().unwrap_or(&vec![]),
                    &self.analysis_summary,
                    &self.created_at,
                    &self.updated_at,
                ],
            )
            .await?;

        let ticket_id = row.get("id");

        // Index to ElasticSearch
        if let Err(e) = self.index_to_es().await {
            log::error!("Failed to index ticket to ElasticSearch: {}", e);
        }

        Ok(ticket_id)
    }

    /// Associates an email with the ticket.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `email_id` - Email to associate
    ///
    /// # Returns
    /// * `Result<(), TicketError>` - Success or error
    pub async fn add_email(&self, pool: &Pool, email_id: &Uuid) -> Result<(), TicketError> {
        let client = pool.get().await?;

        // First verify the email exists
        let email_exists = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM emails WHERE id = $1)",
                &[&email_id],
            )
            .await?
            .get::<_, bool>(0);

        // If the email does not exist, return an error
        if !email_exists {
            return Err(TicketError::Validation(format!(
                "Email {} does not exist",
                email_id
            )));
        }

        // Insert the email association into the database
        client
            .execute(
                "INSERT INTO email_tickets (email_id, ticket_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&email_id, &self.id],
            )
            .await?;

        // Update ElasticSearch
        let es_client = ESClient::new()
            .await
            .map_err(|e| TicketError::Pool(e.to_string()))?;

        // Update the ElasticSearch document with the new email association
        if let Err(e) = es_client
            .update_document(
                "tickets",
                &self.id.to_string(),
                &json!({
                    "email_ids": self.email_ids
                }),
            )
            .await
        {
            log::error!("Failed to update ticket in ElasticSearch: {}", e);
        }

        Ok(())
    }

    /// Removes an email association.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `email_id` - Email to disassociate
    ///
    /// # Returns
    /// * `Result<(), TicketError>` - Success or error
    pub async fn remove_email(&self, pool: &Pool, email_id: &Uuid) -> Result<(), TicketError> {
        let client = pool.get().await?;

        // Delete the email association from the database
        let result = client
            .execute(
                "DELETE FROM email_tickets WHERE email_id = $1 AND ticket_id = $2",
                &[&email_id, &self.id],
            )
            .await?;

        // If the email association was not found, return an error
        if result == 0 {
            return Err(TicketError::Validation(format!(
                "Email {} is not linked to ticket {}",
                email_id, self.id
            )));
        }

        // Update ElasticSearch
        let es_client = ESClient::new()
            .await
            .map_err(|e| TicketError::Pool(e.to_string()))?;

        // Update the ElasticSearch document to remove the email association
        if let Err(e) = es_client
            .update_document(
                "tickets",
                &self.id.to_string(),
                &json!({
                    "email_ids": self.email_ids
                }),
            )
            .await
        {
            log::error!("Failed to update ticket in ElasticSearch: {}", e);
        }

        Ok(())
    }

    /// Updates ticket processing status.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `status` - New status
    ///
    /// # Returns
    /// * `Result<(), TicketError>` - Success or error
    pub async fn update_status(
        &mut self,
        pool: &Pool,
        status: TicketStatus,
    ) -> Result<(), TicketError> {
        log::info!("Updating ticket {} status to {:?}", self.id, status);
        let client = pool.get().await?;

        // Update the ticket status in the database
        let row = client
            .query_one(
                "UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
                &[&status.to_string(), &self.id],
            )
            .await?;

        // Update ElasticSearch
        let es_client = ESClient::new()
            .await
            .map_err(|e| TicketError::Pool(e.to_string()))?;

        // Update the ElasticSearch document with the new status
        if let Err(e) = es_client
            .update_document(
                "tickets",
                &self.id.to_string(),
                &json!({
                    "status": status.to_string(),
                    "updated_at": row.get::<_, DateTime<Utc>>("updated_at")
                }),
            )
            .await
        {
            log::error!("Failed to update ticket in ElasticSearch: {}", e);
        }

        self.status = status;
        self.updated_at = row.get("updated_at");
        Ok(())
    }

    /// Retrieves associated email IDs.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Vec<Uuid>, TicketError>` - Email IDs or error
    pub async fn get_emails(&self, pool: &Pool) -> Result<Vec<Uuid>, TicketError> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT email_id FROM email_tickets WHERE ticket_id = $1",
                &[&self.id],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get("email_id")).collect())
    }

    /// Retrieves all tickets with associations.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Vec<Ticket>, TicketError>` - All tickets or error
    pub async fn list_all(pool: &Pool) -> Result<Vec<Ticket>, TicketError> {
        log::info!("Fetching all tickets");
        let client = pool.get().await?;

        // Fetch all tickets with associated emails
        let rows = client
            .query(
                "SELECT t.*, COALESCE(array_agg(et.email_id) FILTER (WHERE et.email_id IS NOT NULL), ARRAY[]::uuid[]) as email_ids 
                 FROM tickets t 
                 LEFT JOIN email_tickets et ON t.id = et.ticket_id 
                 GROUP BY t.id 
                 ORDER BY t.created_at DESC",
                &[],
            )
            .await?;

        // Convert rows to tickets
        let tickets: Vec<Ticket> = rows
            .iter()
            .map(|row| {
                let mut ticket = Ticket::from(row.clone());
                ticket.email_ids = row.get("email_ids");
                ticket
            })
            .collect();

        Ok(tickets)
    }

    /// Finds a specific ticket by ID.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Ticket identifier
    ///
    /// # Returns
    /// * `Result<Option<Ticket>, TicketError>` - Found ticket or error
    pub async fn find_by_id(pool: &Pool, id: Uuid) -> Result<Option<Ticket>, TicketError> {
        log::info!("Finding ticket {}", id);
        let client = pool.get().await?;

        // Fetch the ticket with associated emails
        let row = client
            .query_opt(
                "SELECT t.*, COALESCE(array_agg(et.email_id) FILTER (WHERE et.email_id IS NOT NULL), ARRAY[]::uuid[]) as email_ids 
                 FROM tickets t 
                 LEFT JOIN email_tickets et ON t.id = et.ticket_id 
                 WHERE t.id = $1 
                 GROUP BY t.id",
                &[&id],
            )
            .await?;

        // Convert row to ticket
        Ok(row.map(|r| {
            let mut ticket = Ticket::from(r.clone());
            ticket.email_ids = r.get("email_ids");
            ticket
        }))
    }

    /// Save ticket to database using a specific client (for transactions)
    pub async fn save_with_client(
        &self,
        client: &tokio_postgres::Transaction<'_>,
    ) -> Result<Uuid, TicketError> {
        log::info!("Saving ticket {}", self.id);

        // Insert the ticket into the database
        let stmt = client
            .prepare(
                "INSERT INTO tickets (
                    id, ticket_type, status, ip_address, subject, description,
                    confidence_score, identified_threats, extracted_indicators, analysis_summary,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::text[], $9::text[], $10, $11, $12) 
                RETURNING id",
            )
            .await?;

        // Execute the insert statement
        let row = client
            .query_one(
                &stmt,
                &[
                    &self.id,
                    &self.ticket_type.to_string(),
                    &self.status.to_string(),
                    &self.ip_address,
                    &self.subject,
                    &self.description,
                    &self.confidence_score,
                    &self.identified_threats.as_ref().unwrap_or(&vec![]),
                    &self.extracted_indicators.as_ref().unwrap_or(&vec![]),
                    &self.analysis_summary,
                    &self.created_at,
                    &self.updated_at,
                ],
            )
            .await?;

        Ok(row.get("id"))
    }

    /// Add an email to this ticket using a specific client (for transactions)
    pub async fn add_email_with_client(
        &self,
        client: &tokio_postgres::Transaction<'_>,
        email_id: &Uuid,
    ) -> Result<(), TicketError> {
        // First verify the email exists
        let email_exists = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM emails WHERE id = $1)",
                &[&email_id],
            )
            .await?
            .get::<_, bool>(0);

        // If the email does not exist, return an error
        if !email_exists {
            return Err(TicketError::Validation(format!(
                "Email {} does not exist",
                email_id
            )));
        }

        // Insert the email association into the database
        client
            .execute(
                "INSERT INTO email_tickets (email_id, ticket_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&email_id, &self.id],
            )
            .await?;

        Ok(())
    }

    /// Searches tickets using Elasticsearch.
    ///
    /// # Arguments
    /// * `options` - Search criteria and filters
    ///
    /// # Returns
    /// * `Result<SearchResponse, TicketError>` - Search results or error
    pub async fn search(options: SearchOptions) -> Result<SearchResponse, TicketError> {
        let client = ESClient::new().await?;

        // Build the search query
        let mut query = json!({
            "bool": {
                "must": [{
                    "multi_match": {
                        "query": options.query,
                        "fields": ["subject^2", "description", "ip_address"],
                        "fuzziness": "AUTO"
                    }
                }],
                "filter": []
            }
        });

        // Add filters if provided
        if let Some(filters) = options.filters {
            if let Some(status) = filters.status {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term": {"status.keyword": status.to_string()}}));
            }

            // Add ticket type filter if provided
            if let Some(ticket_type) = filters.ticket_type {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term": {"ticket_type.keyword": ticket_type.to_string()}}));
            }

            // Add email filter if provided
            if let Some(has_emails) = filters.has_emails {
                query["bool"]["filter"]
                    .as_array_mut()
                    .unwrap()
                    .push(if has_emails {
                        json!({
                            "exists": {
                                "field": "email_ids"
                            }
                        })
                    } else {
                        json!({
                            "bool": {
                                "must_not": {
                                    "exists": {
                                        "field": "email_ids"
                                    }
                                }
                            }
                        })
                    });
            }
        }

        log::debug!(
            "Search query: {}",
            serde_json::to_string_pretty(&query).unwrap()
        );

        // Build the search body
        let search_body = json!({
            "query": query,
            "sort": [{ "created_at": { "order": "desc" } }],
            "from": options.from.unwrap_or(0),
            "size": options.size.unwrap_or(50)
        });

        // Execute the search
        let result = client.search::<Ticket>("tickets", search_body).await?;

        Ok(SearchResponse {
            hits: result.hits,
            total: result.total,
        })
    }

    /// Indexes ticket in Elasticsearch.
    ///
    /// # Returns
    /// * `Result<(), TicketError>` - Success or error
    pub async fn index_to_es(&self) -> Result<(), TicketError> {
        // Create a new Elasticsearch client
        let client = ESClient::new()
            .await
            .map_err(|e| TicketError::Pool(e.to_string()))?;

        // Build the document to index
        let document = json!({
            "id": self.id,
            "ticket_type": self.ticket_type.to_string(),
            "status": self.status.to_string(),
            "ip_address": self.ip_address,
            "subject": self.subject,
            "description": self.description,
            "confidence_score": self.confidence_score,
            "identified_threats": self.identified_threats,
            "extracted_indicators": self.extracted_indicators,
            "analysis_summary": self.analysis_summary,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "email_ids": self.email_ids
        });

        // Index the document
        client
            .index_document("tickets", &self.id.to_string(), &document)
            .await
            .map_err(|e| TicketError::Pool(e.to_string()))
    }
}
