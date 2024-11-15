use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    pub ip_address: Option<String>,
    pub email_id: String,
    pub subject: String,
    pub description: String,
    pub confidence_score: Option<f32>,
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
