//! LLM integration module for security threat analysis.
//!
//! This module provides functionality to analyze text for security threats using
//! Large Language Models (LLMs). It handles:
//! - Text analysis for security threats
//! - LLM client management
//! - Response parsing
//! - Structured threat analysis

/// Core analysis functionality
mod analyzer;
/// LLM client implementation and connection management
mod client;
/// Response parsing and data extraction
mod parser;
/// Threat analysis data structures and processing
mod threat_analysis;

// Re-export the main analysis function for external use
pub use analyzer::analyze_threat;
