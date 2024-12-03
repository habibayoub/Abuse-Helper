use elasticsearch::indices::{IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts};
use elasticsearch::{
    http::transport::{BuildError, SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch, Error as ElasticError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use url::Url;

/// Custom error types for Elasticsearch operations.
///
/// Provides comprehensive error handling for all Elasticsearch-related operations,
/// including connection, query, and index management errors.
///
/// # Error Categories
/// - Connection errors (URL, environment)
/// - Operation errors (index, search, delete)
/// - Data errors (serialization, parsing)
/// - Configuration errors (invalid input)
///
/// # Error Propagation
/// All errors are properly wrapped and context is preserved
/// through the error chain.
#[derive(Debug, thiserror::Error)]
pub enum ESError {
    /// Wrapper for native Elasticsearch errors
    #[error("Elasticsearch error: {0}")]
    Elastic(#[from] ElasticError),
    /// Environment variable configuration errors
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),
    /// URL parsing errors for ES connection
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Document or index not found errors
    #[error("Not found")]
    NotFound,
    /// Invalid input parameters or configuration
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Conversion implementation for Elasticsearch build errors
impl From<BuildError> for ESError {
    fn from(error: BuildError) -> Self {
        ESError::InvalidInput(error.to_string())
    }
}

/// Generic search result container.
///
/// Wraps search results from Elasticsearch queries with pagination information.
///
/// # Type Parameters
/// * `T` - The type of documents being searched for
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult<T> {
    /// Total number of matching documents
    pub total: u64,
    /// Vector of matching documents
    pub hits: Vec<T>,
}

/// Elasticsearch client wrapper.
///
/// Provides a high-level interface for interacting with Elasticsearch,
/// including document CRUD operations and index management.
///
/// # Features
/// - Connection pooling
/// - Proxy configuration
/// - Custom analyzers
/// - Index templates
/// - Bulk operations
/// - Type-safe operations
pub struct ESClient {
    /// Native Elasticsearch client instance
    client: Elasticsearch,
}

impl ESClient {
    /// Creates a new Elasticsearch client instance.
    ///
    /// Initializes connection using ELASTICSEARCH_URL environment variable.
    /// Configures single-node connection pool with proxy disabled.
    pub async fn new() -> Result<Self, ESError> {
        let url = env::var("ELASTICSEARCH_URL")?;
        let url = Url::parse(&url)?;

        // Create a single-node connection pool
        let conn_pool = SingleNodeConnectionPool::new(url);
        // Create a transport with disabled proxy
        let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;

        Ok(Self {
            client: Elasticsearch::new(transport),
        })
    }

    /// Indexes a document in Elasticsearch.
    ///
    /// # Arguments
    /// * `index` - Name of the index to store the document in
    /// * `id` - Unique identifier for the document
    /// * `document` - The document to index (must be serializable)
    ///
    /// # Errors
    /// Returns ESError if indexing fails or if the document cannot be serialized
    pub async fn index_document<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: &T,
    ) -> Result<(), ESError> {
        // Send the document to Elasticsearch
        let response = self
            .client
            .index(elasticsearch::IndexParts::IndexId(index, id))
            .body(document)
            .send()
            .await?;

        // Check if the indexing operation was successful
        if !response.status_code().is_success() {
            return Err(ESError::InvalidInput(format!(
                "Failed to index document: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    /// Performs a search query against an index.
    ///
    /// # Arguments
    /// * `index` - Index to search in
    /// * `query` - Elasticsearch query in JSON format
    ///
    /// # Type Parameters
    /// * `T` - Type to deserialize results into
    pub async fn search<T: for<'de> Deserialize<'de>>(
        &self,
        index: &str,
        query: Value,
    ) -> Result<SearchResult<T>, ESError> {
        // Send the search query to Elasticsearch
        let response = self
            .client
            .search(elasticsearch::SearchParts::Index(&[index]))
            .body(query)
            .send()
            .await?;

        // Check if the search operation was successful
        let status = response.status_code().as_u16();
        if status == 404 {
            return Err(ESError::NotFound);
        }

        // Parse the response body
        let response_body = response.json::<Value>().await?;
        // Extract the total number of hits
        let total = response_body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or_default();

        // Extract the hits from the response
        let hits: Vec<T> = response_body["hits"]["hits"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|hit| serde_json::from_value(hit["_source"].clone()).ok())
            .collect();

        Ok(SearchResult { total, hits })
    }

    /// Deletes a document from an index.
    ///
    /// # Arguments
    /// * `index` - Index containing the document
    /// * `id` - ID of the document to delete
    pub async fn delete_document(&self, index: &str, id: &str) -> Result<(), ESError> {
        // Send the delete request to Elasticsearch
        let response = self
            .client
            .delete(elasticsearch::DeleteParts::IndexId(index, id))
            .send()
            .await?;

        // Check if the delete operation was successful
        let status = response.status_code().as_u16();
        if !response.status_code().is_success() && status != 404 {
            return Err(ESError::InvalidInput(format!(
                "Failed to delete document: {}",
                status
            )));
        }

        Ok(())
    }

    /// Updates an existing document or creates it if it doesn't exist.
    ///
    /// # Arguments
    /// * `index` - Target index
    /// * `id` - Document ID
    /// * `document` - Updated document content
    pub async fn update_document<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: &T,
    ) -> Result<(), ESError> {
        // Send the update request to Elasticsearch
        let response = self
            .client
            .update(elasticsearch::UpdateParts::IndexId(index, id))
            .body(json!({
                "doc": document,
                "doc_as_upsert": true
            }))
            .send()
            .await?;

        // Check if the update operation was successful
        if !response.status_code().is_success() {
            return Err(ESError::InvalidInput(format!(
                "Failed to update document: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    /// Ensures an index exists with proper mappings and settings.
    ///
    /// # Arguments
    /// * `index` - Name of the index to create/verify
    ///
    /// # Supported Indices
    /// * `emails` - Email document index with custom analyzer
    /// * `tickets` - Support ticket index with threat analysis fields
    pub async fn ensure_index(&self, index: &str) -> Result<(), ESError> {
        // Check if the index exists
        let exists = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&[index]))
            .send()
            .await?
            .status_code()
            .is_success();

        if !exists {
            // Create the index with the appropriate mapping
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

            // Create the index with the appropriate mapping
            self.client
                .indices()
                .create(IndicesCreateParts::Index(index))
                .body(mapping)
                .send()
                .await?;
        }

        Ok(())
    }

    /// Deletes an index if it exists.
    ///
    /// # Arguments
    /// * `index` - Name of the index to delete
    pub async fn delete_index(&self, index: &str) -> Result<(), ESError> {
        // Send the delete request to Elasticsearch
        let response = self
            .client
            .indices()
            .delete(IndicesDeleteParts::Index(&[index]))
            .send()
            .await?;

        // Check if the delete operation was successful
        if !response.status_code().is_success() && response.status_code().as_u16() != 404 {
            return Err(ESError::InvalidInput(format!(
                "Failed to delete index: {}",
                response.status_code()
            )));
        }

        Ok(())
    }
}
