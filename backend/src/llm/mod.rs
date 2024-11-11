use crate::models::ticket::TicketType;
use kalosm::language::*;
use log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatAnalysis {
    pub threat_type: TicketType,
    pub confidence_score: f32,
    pub identified_threats: Vec<String>,
    pub extracted_indicators: Vec<String>,
    pub summary: String,
}

pub async fn analyze_threat(text: &str) -> Result<ThreatAnalysis, String> {
    let model = match Llama::builder()
        .with_source(LlamaSource::llama_3_2_3b_chat())
        .build()
        .await
    {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to initialize Llama model: {}", e);
            return Err(format!("Failed to initialize Llama model: {}", e));
        }
    };

    let mut chat = Chat::builder(model)
        .with_system_prompt(
            "You are a security threat analyzer. Analyze content for security threats, scams, or abuse. \
             Provide analysis in a structured format."
        )
        .build();

    let prompt = format!(
        r#"Analyze this content for security threats, scams, or abuse:

Content: {}

Classify the type of threat from these categories:
Malware, Phishing, Scam, Spam, DDoS, Botnet, DataBreach, IdentityTheft, 
Ransomware, CyberStalking, IntellectualPropertyTheft, ChildAbuse, 
Harassment, UnauthorizedAccess, Other

Provide:
1. Threat type (from categories above)
2. Confidence score (0-1)
3. List of threat indicators
4. List of extracted information (IPs, URLs, etc)
5. Brief summary

Format as:
Type: [category]
Confidence: [0-1]
Indicators: [list]
Extracted: [list]
Summary: [text]"#,
        text
    );

    let response = chat.add_message(&prompt).all_text().await;

    log::debug!("Raw LLM response: {}", response);

    // If we get an empty response, return a default analysis
    if response.trim().is_empty() {
        log::warn!("Empty response from LLM, using default analysis");
        return Ok(ThreatAnalysis {
            threat_type: TicketType::Other,
            confidence_score: 0.0,
            identified_threats: vec![],
            extracted_indicators: vec![],
            summary: "Analysis failed - no response from model".to_string(),
        });
    }

    parse_llm_response(&response)
}

fn parse_llm_response(response: &str) -> Result<ThreatAnalysis, String> {
    let mut threat_type = TicketType::Other;
    let mut confidence_score = 0.0;
    let mut identified_threats = Vec::new();
    let mut extracted_indicators = Vec::new();
    let mut summary = String::new();

    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("Type:") {
            threat_type = match line.trim_start_matches("Type:").trim() {
                "Malware" => TicketType::Malware,
                "Phishing" => TicketType::Phishing,
                "Scam" => TicketType::Scam,
                "Spam" => TicketType::Spam,
                "DDoS" => TicketType::DDoS,
                "Botnet" => TicketType::Botnet,
                "DataBreach" => TicketType::DataBreach,
                "IdentityTheft" => TicketType::IdentityTheft,
                "Ransomware" => TicketType::Ransomware,
                "CyberStalking" => TicketType::CyberStalking,
                "IntellectualPropertyTheft" => TicketType::IntellectualPropertyTheft,
                "ChildAbuse" => TicketType::ChildAbuse,
                "Harassment" => TicketType::Harassment,
                "UnauthorizedAccess" => TicketType::UnauthorizedAccess,
                _ => TicketType::Other,
            };
        } else if line.starts_with("Confidence:") {
            confidence_score = line
                .trim_start_matches("Confidence:")
                .trim()
                .parse()
                .unwrap_or(0.0);
        } else if line.starts_with("Indicators:") {
            identified_threats = line
                .trim_start_matches("Indicators:")
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else if line.starts_with("Extracted:") {
            extracted_indicators = line
                .trim_start_matches("Extracted:")
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else if line.starts_with("Summary:") {
            summary = line.trim_start_matches("Summary:").trim().to_string();
        }
    }

    Ok(ThreatAnalysis {
        threat_type,
        confidence_score,
        identified_threats,
        extracted_indicators,
        summary,
    })
}
