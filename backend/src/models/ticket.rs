use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

/// Types of security incidents that can be reported
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TicketType {
    Malware,
    Phishing,
    Scam,
    Spam,
    DDoS,
    Botnet,
    DataBreach,
    IdentityTheft,
    Ransomware,
    CyberStalking,
    IntellectualPropertyTheft,
    Harassment,
    UnauthorizedAccess,
    CopyrightViolation,
    BruteForce,
    C2,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TicketStatus {
    Open,
    InProgress,
    Closed,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    pub ip_address: Option<String>,
    pub email_id: String,
    pub subject: String,
    pub description: String,
    pub confidence_score: Option<f64>,
    pub identified_threats: Option<Vec<String>>,
    pub extracted_indicators: Option<Vec<String>>,
    pub analysis_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Row> for Ticket {
    fn from(row: Row) -> Self {
        Ticket {
            id: row.get("id"),
            ticket_type: TicketType::from(row.get::<_, String>("ticket_type")),
            status: TicketStatus::from(row.get::<_, String>("status")),
            ip_address: row.get("ip_address"),
            email_id: row.get("email_id"),
            subject: row.get("subject"),
            description: row.get("description"),
            confidence_score: row.get("confidence_score"),
            identified_threats: row.get("identified_threats"),
            extracted_indicators: row.get("extracted_indicators"),
            analysis_summary: row.get("analysis_summary"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Request payload for creating a new ticket
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTicketRequest {
    pub email_id: String,
    pub subject: String,
    pub description: String,
    pub ticket_type: Option<TicketType>,
    pub confidence_score: Option<f64>,
    pub identified_threats: Option<Vec<String>>,
    pub extracted_indicators: Option<Vec<String>>,
    pub analysis_summary: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl From<deadpool_postgres::PoolError> for TicketError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        TicketError::Pool(error.to_string())
    }
}

impl Ticket {
    /// Create a new ticket
    pub fn new(
        ticket_type: TicketType,
        email_id: String,
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
            email_id,
            subject,
            description,
            confidence_score,
            identified_threats,
            extracted_indicators,
            analysis_summary,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Save ticket to database
    pub async fn save(&self, pool: &Pool) -> Result<Uuid, TicketError> {
        log::info!("Saving ticket {}", self.id);
        let client = pool.get().await?;

        let stmt = client
            .prepare(
                "INSERT INTO tickets (
                    id, ticket_type, status, ip_address, email_id, subject, description,
                    confidence_score, identified_threats, extracted_indicators, analysis_summary,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) 
                RETURNING id",
            )
            .await?;

        client
            .query_one(
                &stmt,
                &[
                    &self.id,
                    &self.ticket_type.to_string(),
                    &self.status.to_string(),
                    &self.ip_address,
                    &self.email_id,
                    &self.subject,
                    &self.description,
                    &self.confidence_score,
                    &self.identified_threats,
                    &self.extracted_indicators,
                    &self.analysis_summary,
                    &self.created_at,
                    &self.updated_at,
                ],
            )
            .await?;

        // Add JIRA ticket creation logic here

        Ok(self.id)
    }

    /// Update ticket status
    pub async fn update_status(
        &mut self,
        pool: &Pool,
        status: TicketStatus,
    ) -> Result<(), TicketError> {
        log::info!("Updating ticket {} status to {:?}", self.id, status);
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
                &[&status.to_string(), &self.id],
            )
            .await?;

        self.status = status;
        self.updated_at = row.get("updated_at");
        Ok(())
    }

    /// List all tickets
    pub async fn list_all(pool: &Pool) -> Result<Vec<Ticket>, TicketError> {
        log::info!("Fetching all tickets");
        let client = pool.get().await?;

        let rows = client
            .query("SELECT * FROM tickets ORDER BY created_at DESC", &[])
            .await?;

        Ok(rows.into_iter().map(Ticket::from).collect())
    }

    /// Find ticket by ID
    pub async fn find_by_id(pool: &Pool, id: Uuid) -> Result<Option<Ticket>, TicketError> {
        log::info!("Finding ticket {}", id);
        let client = pool.get().await?;

        let row = client
            .query_opt("SELECT * FROM tickets WHERE id = $1", &[&id])
            .await?;

        Ok(row.map(Ticket::from))
    }

    /// Add validation method
    pub fn validate(&self) -> Result<(), TicketError> {
        if self.subject.is_empty() {
            return Err(TicketError::Validation("Subject cannot be empty".into()));
        }
        if self.description.is_empty() {
            return Err(TicketError::Validation(
                "Description cannot be empty".into(),
            ));
        }
        if self.email_id.is_empty() {
            return Err(TicketError::Validation("Email ID cannot be empty".into()));
        }
        Ok(())
    }
}
