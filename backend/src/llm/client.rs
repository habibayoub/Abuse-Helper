use log;
use ollama_rs::Ollama;
use std::env;

/// A client wrapper for interacting with the Ollama LLM service.
///
/// Provides a simplified interface for making requests to a local Ollama instance,
/// managing the connection and error handling.
///
/// # Example
/// ```
/// let client = LLMClient::new()?;
/// let response = client.get_client().generate(request).await?;
/// ```
pub struct LLMClient {
    /// Internal Ollama client instance
    client: Ollama,
}

impl LLMClient {
    /// Creates a new LLMClient instance.
    ///
    /// Initializes connection to Ollama service using the OLLAMA_URL environment variable.
    ///
    /// # Returns
    /// * `Ok(LLMClient)` - Successfully initialized client
    /// * `Err(String)` - Connection or configuration error message
    ///
    /// # Errors
    /// Returns error if:
    /// - OLLAMA_URL environment variable is not set
    /// - Cannot establish connection to Ollama service
    pub fn new() -> Result<Self, String> {
        let client = Ollama::try_new(env::var("OLLAMA_URL").expect("OLLAMA_URL must be set"))
            .map_err(|e| {
                log::error!("Failed to create Ollama instance: {}", e);
                format!("Failed to create Ollama instance: {}", e)
            })?;

        Ok(Self { client })
    }

    /// Returns a reference to the internal Ollama client.
    ///
    /// Used to access Ollama-specific functionality not wrapped by this struct.
    pub fn get_client(&self) -> &Ollama {
        &self.client
    }
}
