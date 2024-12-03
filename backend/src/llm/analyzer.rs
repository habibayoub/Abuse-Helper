use super::{client::LLMClient, parser, threat_analysis::ThreatAnalysis};
use log;
use ollama_rs::generation::completion::request::GenerationRequest;

/// Performs security threat analysis on provided text using LLM.
///
/// Uses a local LLaMA model to analyze text for potential security threats,
/// extracting indicators like URLs, IPs, and suspicious patterns.
///
/// # Arguments
/// * `text` - Content to analyze
///
/// # Returns
/// * `ThreatAnalysis` - Structured analysis results
/// * `String` - Error message if analysis fails
///
/// # Example
/// ```
/// let analysis = analyze_threat("suspicious content").await?;
/// println!("Threat type: {}", analysis.threat_type);
/// ```
pub async fn analyze_threat(text: &str) -> Result<ThreatAnalysis, String> {
    // Initialize LLM client
    let client = LLMClient::new()?;

    // Define system prompt for threat analysis
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

    // Configure and execute LLM request
    let request = GenerationRequest::new("llama3.2:1b".to_string(), prompt);

    // Process response, logging any errors
    let response = client.get_client().generate(request).await.map_err(|e| {
        log::error!("Failed to generate response: {}", e);
        format!("Failed to generate response: {}", e)
    })?;

    log::info!("Raw model response: {}", response.response);
    parser::parse_llm_response(&response.response)
}
