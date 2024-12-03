use crate::models::ticket::TicketType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request structure for associating an email with a ticket.
///
/// Used when adding existing emails to tickets or creating
/// email-ticket relationships.
///
/// # Fields
/// * `email_id` - Unique identifier of the email to be added
#[derive(Deserialize)]
pub struct AddEmailRequest {
    pub email_id: Uuid,
}

/// Response structure for ticket creation operations.
///
/// Provides feedback about the ticket creation process, including
/// successful and failed email associations.
///
/// # Fields
/// * `ticket_id` - Unique identifier of the created ticket
/// * `linked_emails` - List of successfully linked email IDs
/// * `failed_emails` - List of emails that failed to link, with error messages
#[derive(Serialize)]
pub struct CreateTicketResponse {
    pub ticket_id: Uuid,
    pub linked_emails: Vec<String>,
    pub failed_emails: Vec<(String, String)>, // (email_id, error_message)
}

/// Request payload for creating a new ticket.
///
/// Contains all necessary information for ticket creation, including
/// optional threat analysis data and related email references.
///
/// # Fields
/// * `email_ids` - List of related email identifiers
/// * `subject` - Ticket subject line
/// * `description` - Detailed ticket description
/// * `ticket_type` - Optional classification of the ticket
/// * `confidence_score` - Optional threat confidence rating (0.0-1.0)
/// * `identified_threats` - Optional list of detected threats
/// * `extracted_indicators` - Optional list of security indicators
/// * `analysis_summary` - Optional threat analysis summary
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
    /// Validates the ticket creation request.
    ///
    /// Performs comprehensive validation of all required and optional fields:
    /// - Ensures subject and description are not empty
    /// - Verifies at least one email ID is provided
    /// - Validates confidence score range if present
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(TicketError)` with specific validation error message
    ///
    /// # Validation Rules
    /// - Subject must not be empty
    /// - Description must not be empty
    /// - At least one email ID required
    /// - Confidence score must be between 0.0 and 1.0 if provided
    pub fn validate(&self) -> Result<(), crate::models::ticket::TicketError> {
        // Validate the subject
        if self.subject.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "Subject cannot be empty".into(),
            ));
        }
        // Validate the description
        if self.description.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "Description cannot be empty".into(),
            ));
        }
        // Validate the email IDs
        if self.email_ids.is_empty() {
            return Err(crate::models::ticket::TicketError::Validation(
                "At least one email ID must be provided".into(),
            ));
        }
        // Validate the confidence score
        if let Some(score) = self.confidence_score {
            if !(0.0..=1.0).contains(&score) {
                return Err(crate::models::ticket::TicketError::Validation(
                    "Confidence score must be between 0 and 1".into(),
                ));
            }
        }
        // If all validation checks pass, return Ok
        Ok(())
    }
}
