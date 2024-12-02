use super::threat_analysis::ThreatAnalysis;
use log;

/// Parses and validates the LLM's response into a structured ThreatAnalysis object.
///
/// Attempts to parse the raw response string from the language model into a
/// strongly-typed ThreatAnalysis structure. Handles various error cases gracefully
/// by returning default error states rather than failing completely.
///
/// # Arguments
/// * `response` - Raw string response from the language model, expected to be in JSON format
///
/// # Returns
/// * `Result<ThreatAnalysis, String>` - Either a parsed ThreatAnalysis object or an error message
///   Note: Even in error cases, returns Ok(ThreatAnalysis) with default error state
///
/// # Examples
/// ```
/// let response = r#"{"threat_type": "MALWARE", "confidence_score": 0.95}"#;
/// let analysis = parse_llm_response(response)?;
/// ```
///
/// # Error Handling
/// - Empty responses return a default error state
/// - Invalid JSON returns a default error state with parsing failure message
/// - All errors are logged for debugging purposes
pub fn parse_llm_response(response: &str) -> Result<ThreatAnalysis, String> {
    // Check for empty response, return default error if true
    if response.trim().is_empty() {
        log::warn!("Empty response from model");
        return Ok(ThreatAnalysis::default_error(
            "Analysis failed - no response from model",
        ));
    }

    // Attempt to parse the response as JSON
    match serde_json::from_str::<ThreatAnalysis>(response) {
        Ok(analysis) => {
            log::info!("Successfully parsed JSON response: {:?}", analysis);
            Ok(analysis)
        }
        Err(e) => {
            log::error!("Failed to parse response as JSON: {}", e);
            Ok(ThreatAnalysis::default_error(
                "Failed to parse analysis response",
            ))
        }
    }
}
