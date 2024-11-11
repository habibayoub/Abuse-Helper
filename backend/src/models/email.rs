use chrono::{DateTime, Utc};
use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

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
