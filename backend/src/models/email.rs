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
#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub received_at: DateTime<Utc>,
}

impl From<Row> for Email {
    fn from(row: Row) -> Self {
        Email {
            id: row.get("id"),
            from: row.get("from"),
            to: row.get("to"),
            subject: row.get("subject"),
            body: row.get("body"),
            received_at: row.get("received_at"),
        }
    }
}
