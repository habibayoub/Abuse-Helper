use elasticsearch::indices::{IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts};
use elasticsearch::{
    http::transport::{BuildError, SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch, Error as ElasticError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum ESError {
    #[error("Elasticsearch error: {0}")]
    Elastic(#[from] ElasticError),
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<BuildError> for ESError {
    fn from(error: BuildError) -> Self {
        ESError::InvalidInput(error.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub total: u64,
    pub hits: Vec<T>,
}

pub struct ESClient {
    client: Elasticsearch,
}

impl ESClient {
    pub async fn new() -> Result<Self, ESError> {
        let url = env::var("ELASTICSEARCH_URL")?;
        let url = Url::parse(&url)?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;

        Ok(Self {
            client: Elasticsearch::new(transport),
        })
    }

    pub async fn index_document<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: &T,
    ) -> Result<(), ESError> {
        let response = self
            .client
            .index(elasticsearch::IndexParts::IndexId(index, id))
            .body(document)
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(ESError::InvalidInput(format!(
                "Failed to index document: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    pub async fn search<T: for<'de> Deserialize<'de>>(
        &self,
        index: &str,
        query: Value,
    ) -> Result<SearchResult<T>, ESError> {
        let response = self
            .client
            .search(elasticsearch::SearchParts::Index(&[index]))
            .body(query)
            .send()
            .await?;

        let status = response.status_code().as_u16();
        if status == 404 {
            return Err(ESError::NotFound);
        }

        let response_body = response.json::<Value>().await?;
        let total = response_body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or_default();

        let hits: Vec<T> = response_body["hits"]["hits"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|hit| serde_json::from_value(hit["_source"].clone()).ok())
            .collect();

        Ok(SearchResult { total, hits })
    }

    pub async fn delete_document(&self, index: &str, id: &str) -> Result<(), ESError> {
        let response = self
            .client
            .delete(elasticsearch::DeleteParts::IndexId(index, id))
            .send()
            .await?;

        let status = response.status_code().as_u16();
        if !response.status_code().is_success() && status != 404 {
            return Err(ESError::InvalidInput(format!(
                "Failed to delete document: {}",
                status
            )));
        }

        Ok(())
    }

    pub async fn update_document<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: &T,
    ) -> Result<(), ESError> {
        let response = self
            .client
            .update(elasticsearch::UpdateParts::IndexId(index, id))
            .body(json!({
                "doc": document,
                "doc_as_upsert": true
            }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(ESError::InvalidInput(format!(
                "Failed to update document: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    pub async fn ensure_index(&self, index: &str) -> Result<(), ESError> {
        let exists = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&[index]))
            .send()
            .await?
            .status_code()
            .is_success();

        if !exists {
            let mapping = match index {
                "emails" => json!({
                    "settings": {
                        "analysis": {
                            "analyzer": {
                                "email_analyzer": {
                                    "type": "custom",
                                    "tokenizer": "standard",
                                    "filter": ["lowercase", "stop", "snowball"]
                                }
                            }
                        }
                    },
                    "mappings": {
                        "properties": {
                            "id": { "type": "keyword" },
                            "sender": { "type": "text", "analyzer": "email_analyzer" },
                            "recipients": { "type": "text", "analyzer": "email_analyzer" },
                            "subject": { "type": "text", "analyzer": "email_analyzer" },
                            "body": { "type": "text", "analyzer": "email_analyzer" },
                            "received_at": { "type": "date" },
                            "analyzed": { "type": "boolean" },
                            "is_sent": { "type": "boolean" },
                            "ticket_ids": { "type": "keyword" }
                        }
                    }
                }),
                "tickets" => json!({
                    "settings": {
                        "analysis": {
                            "analyzer": {
                                "ticket_analyzer": {
                                    "type": "custom",
                                    "tokenizer": "standard",
                                    "filter": ["lowercase", "stop", "snowball"]
                                }
                            }
                        }
                    },
                    "mappings": {
                        "properties": {
                            "id": { "type": "keyword" },
                            "ticket_type": { "type": "keyword" },
                            "status": { "type": "keyword" },
                            "ip_address": { "type": "ip" },
                            "subject": { "type": "text", "analyzer": "ticket_analyzer" },
                            "description": { "type": "text", "analyzer": "ticket_analyzer" },
                            "confidence_score": { "type": "float" },
                            "identified_threats": { "type": "keyword" },
                            "extracted_indicators": { "type": "keyword" },
                            "analysis_summary": { "type": "text", "analyzer": "ticket_analyzer" },
                            "created_at": { "type": "date" },
                            "updated_at": { "type": "date" },
                            "email_ids": { "type": "keyword" }
                        }
                    }
                }),
                _ => return Err(ESError::InvalidInput("Invalid index name".to_string())),
            };

            self.client
                .indices()
                .create(IndicesCreateParts::Index(index))
                .body(mapping)
                .send()
                .await?;
        }

        Ok(())
    }

    pub async fn delete_index(&self, index: &str) -> Result<(), ESError> {
        let response = self
            .client
            .indices()
            .delete(IndicesDeleteParts::Index(&[index]))
            .send()
            .await?;

        if !response.status_code().is_success() && response.status_code().as_u16() != 404 {
            return Err(ESError::InvalidInput(format!(
                "Failed to delete index: {}",
                response.status_code()
            )));
        }

        Ok(())
    }
}
