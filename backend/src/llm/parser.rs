use super::threat_analysis::ThreatAnalysis;
use log;

pub fn parse_llm_response(response: &str) -> Result<ThreatAnalysis, String> {
    if response.trim().is_empty() {
        log::warn!("Empty response from model");
        return Ok(ThreatAnalysis::default_error(
            "Analysis failed - no response from model",
        ));
    }

    // Try to parse as JSON
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
