use crate::models::ticket::TicketType;
use serde::{Deserialize, Serialize};

/// Represents the results of a security threat analysis performed by the LLM.
///
/// This struct encapsulates the complete analysis output, including threat classification,
/// confidence levels, and detailed findings from the analyzed content.
#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatAnalysis {
    /// The classified type of the threat, mapped to corresponding ticket categories
    pub threat_type: TicketType,

    /// Confidence level of the threat analysis, ranging from 0.0 to 1.0
    /// Higher values indicate greater confidence in the analysis results
    pub confidence_score: f64,

    /// List of specific threats identified during analysis
    /// Contains detailed descriptions of each detected threat
    pub identified_threats: Vec<String>,

    /// Collection of extracted suspicious indicators like URLs, IPs, or file hashes
    /// These are concrete artifacts found in the analyzed content
    pub extracted_indicators: Vec<String>,

    /// Comprehensive summary of the threat analysis findings
    /// Provides a human-readable explanation of the analysis results
    pub summary: String,
}

impl ThreatAnalysis {
    /// Creates a default ThreatAnalysis instance for error cases.
    ///
    /// # Arguments
    /// * `message` - Error message to include in the summary
    ///
    /// # Returns
    /// A ThreatAnalysis instance with default values and the provided error message
    ///
    /// # Example
    /// ```
    /// let error_analysis = ThreatAnalysis::default_error("Analysis failed due to invalid input");
    /// ```
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
