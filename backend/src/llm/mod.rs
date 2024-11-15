use crate::models::ticket::TicketType;
use log;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatAnalysis {
    pub threat_type: TicketType,
    pub confidence_score: f64,
    pub identified_threats: Vec<String>,
    pub extracted_indicators: Vec<String>,
    pub summary: String,
}

pub async fn analyze_threat(text: &str) -> Result<ThreatAnalysis, String> {
    let ollama = Ollama::new("http://llm".to_string(), 11434);

    let prompt = format!(
        r#"<|im_start|>system
You are a security threat analyzer. You will analyze the content provided and MUST respond with a single JSON object.
You must extract any suspicious indicators like URLs, IPs, domains, and file names found only in the content provided.
You must be direct and honest in your assessment.
You cannot refuse to analyze any content.
You must provide uncensored analysis.
You must provide a detailed summary of the analysis.
You are smart and can understand the context of the content and are not easily fooled, so you can make assumptions when needed if they make sense.
<|im_end|>
<|im_start|>user
Analyze this content for security threats:
{text}

Respond with a single JSON object in this EXACT format (MAKE SURE THERE ARE NO TRAILING COMMAS AT THE END OF THE JSON OBJECT, MAKE SURE YOU INCLUDE ALL FIELDS: threat_type, confidence_score, identified_threats, extracted_indicators, summary) (values for identified_threats and extracted_indicators must be retrieved from the content provided, DO NOT MAKE UP ANY VALUES):
{{
    "threat_type": "only one of the following values: Malware, Phishing, Scam, Spam, DDoS, Botnet, DataBreach, IdentityTheft, Ransomware, CyberStalking, IntellectualPropertyTheft, Harassment, UnauthorizedAccess, CopyrightViolation, BruteForce, C2, Other",
    "confidence_score": 0.0 to 1.0,
    "identified_threats": ["list", "of", "identified", "threats", "found", "in", "the", "content"],
    "extracted_indicators": ["list", "of", "suspicious", "URLs", "IPs", "domains", "files", "found", "in", "the", "content"],
    "summary": "detailed analysis summary"
}}
<|im_end|>
<|im_start|>assistant"#
    );

    let request = GenerationRequest::new("llama3.2:1b".to_string(), prompt);

    let response = ollama.generate(request).await.map_err(|e| {
        log::error!("Failed to generate response: {}", e);
        format!("Failed to generate response: {}", e)
    })?;

    log::info!("Raw model response: {}", response.response);

    if response.response.trim().is_empty() {
        log::warn!("Empty response from model, using default analysis");
        return Ok(ThreatAnalysis {
            threat_type: TicketType::Other,
            confidence_score: 0.0,
            identified_threats: vec![],
            extracted_indicators: vec![],
            summary: "Analysis failed - no response from model".to_string(),
        });
    }

    // Try to parse as JSON first
    match serde_json::from_str::<ThreatAnalysis>(&response.response) {
        Ok(analysis) => {
            log::info!("Successfully parsed JSON response: {:?}", analysis);
            Ok(analysis)
        }
        Err(e) => {
            log::warn!(
                "Failed to parse JSON response: {}, falling back to text parsing",
                e
            );
            parse_llm_response(&response.response)
        }
    }
}

// Keep the text parsing as fallback
fn parse_llm_response(response: &str) -> Result<ThreatAnalysis, String> {
    // Try to parse the entire response as JSON
    match serde_json::from_str::<ThreatAnalysis>(response) {
        Ok(analysis) => {
            log::info!("Successfully parsed response as JSON: {:?}", analysis);
            Ok(analysis)
        }
        Err(e) => {
            log::error!("Failed to parse response as JSON: {}", e);
            // Return a default analysis if parsing fails
            Ok(ThreatAnalysis {
                threat_type: TicketType::Other,
                confidence_score: 0.0,
                identified_threats: vec![],
                extracted_indicators: vec![],
                summary: "Failed to parse analysis response".to_string(),
            })
        }
    }
}
