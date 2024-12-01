use crate::models::ticket::TicketType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatAnalysis {
    pub threat_type: TicketType,
    pub confidence_score: f64,
    pub identified_threats: Vec<String>,
    pub extracted_indicators: Vec<String>,
    pub summary: String,
}

impl ThreatAnalysis {
    pub fn default_error(message: &str) -> Self {
        Self {
            threat_type: TicketType::Other,
            confidence_score: 0.0,
            identified_threats: vec![],
            extracted_indicators: vec![],
            summary: message.to_string(),
        }
    }
}
