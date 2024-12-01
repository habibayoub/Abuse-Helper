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

    /// Save ticket to database
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

    /// Add an email to this ticket
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

        if !email_exists {
            return Err(TicketError::Validation(format!(
                "Email {} does not exist",
                email_id
            )));
        }

        client
            .execute(
                "INSERT INTO email_tickets (email_id, ticket_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&email_id, &self.id],
            )
            .await?;

        Ok(())
    }

    /// Remove an email from this ticket
    pub async fn remove_email(&self, pool: &Pool, email_id: &Uuid) -> Result<(), TicketError> {
        let client = pool.get().await?;

        let result = client
            .execute(
                "DELETE FROM email_tickets WHERE email_id = $1 AND ticket_id = $2",
                &[&email_id, &self.id],
            )
            .await?;

        if result == 0 {
            return Err(TicketError::Validation(format!(
                "Email {} is not linked to ticket {}",
                email_id, self.id
            )));
        }

        Ok(())
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

    /// Get all emails associated with this ticket
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

    /// List all tickets with their associated emails
    pub async fn list_all(pool: &Pool) -> Result<Vec<Ticket>, TicketError> {
        log::info!("Fetching all tickets");
        let client = pool.get().await?;

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

    /// Find ticket by ID with associated emails
    pub async fn find_by_id(pool: &Pool, id: Uuid) -> Result<Option<Ticket>, TicketError> {
        log::info!("Finding ticket {}", id);
        let client = pool.get().await?;

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

        if !email_exists {
            return Err(TicketError::Validation(format!(
                "Email {} does not exist",
                email_id
            )));
        }

        client
            .execute(
                "INSERT INTO email_tickets (email_id, ticket_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&email_id, &self.id],
            )
            .await?;

        Ok(())
    }
}
