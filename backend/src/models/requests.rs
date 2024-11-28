use crate::models::ticket::TicketType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AddEmailRequest {
    pub email_id: Uuid,
}

#[derive(Serialize)]
pub struct CreateTicketResponse {
    pub ticket_id: Uuid,
    pub linked_emails: Vec<String>,
    pub failed_emails: Vec<(String, String)>, // (email_id, error_message)
}

/// Request payload for creating a new ticket
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTicketRequest {
    pub email_ids: Vec<Uuid>,
    pub subject: String,
    pub description: String,
    pub ticket_type: Option<TicketType>,
    pub confidence_score: Option<f64>,
    pub identified_threats: Option<Vec<String>>,
    pub extracted_indicators: Option<Vec<String>>,
    pub analysis_summary: Option<String>,
}

impl CreateTicketRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), crate::models::ticket::TicketError> {
        if self.subject.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "Subject cannot be empty".into(),
            ));
        }
        if self.description.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "Description cannot be empty".into(),
            ));
        }
        if self.email_ids.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "At least one email ID must be provided".into(),
            ));
        }
        if let Some(score) = self.confidence_score {
            if !(0.0..=1.0).contains(&score) {
                return Err(crate::models::ticket::TicketError::Validation(
                    "Confidence score must be between 0 and 1".into(),
                ));
            }
        }
        Ok(())
    }
}
