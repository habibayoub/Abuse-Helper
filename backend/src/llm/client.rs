use log;
use ollama_rs::Ollama;
use std::env;

pub struct LLMClient {
    client: Ollama,
}

impl LLMClient {
    pub fn new() -> Result<Self, String> {
        let client = Ollama::try_new(env::var("OLLAMA_URL").expect("OLLAMA_URL must be set"))
            .map_err(|e| {
                log::error!("Failed to create Ollama instance: {}", e);
                format!("Failed to create Ollama instance: {}", e)
            })?;
        
        Ok(Self { client })
    }

    pub fn get_client(&self) -> &Ollama {
        &self.client
    }
}
