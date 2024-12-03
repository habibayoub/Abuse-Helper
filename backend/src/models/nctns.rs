use chrono::{DateTime, Utc};
use tokio_postgres::{Error, GenericClient, Row};
use uuid::Uuid;

/// Network and Cyber Threat Notification System Record
///
/// Represents a comprehensive security event or threat notification.
///
/// # Fields
/// * `uuid` - Unique identifier for the notification
/// * `source_time` - Original event timestamp from source
/// * `time` - Event processing timestamp
/// * `ip` - Source IP address
/// * `reverse_dns` - Reverse DNS lookup result
/// * `domain_name` - Associated domain name
/// * `asn` - Autonomous System Number
/// * `as_name` - Autonomous System Name
/// * `category` - Threat category classification
/// * `type_` - Specific threat type
/// * `malware_family` - Associated malware family if applicable
/// * `vulnerability` - Related vulnerability identifier
/// * `tag` - Custom classification tag
/// * `source_name` - Notification source identifier
/// * `comment` - Additional notes or comments
/// * `description` - Detailed threat description
/// * `description_url` - Reference URL for threat details
/// * `destination_ip` - Target IP address
/// * `destination_port` - Target port number
/// * `port` - Source port number
/// * `protocol` - Application protocol
/// * `transport_protocol` - Network transport protocol
/// * `http_request` - HTTP request details if applicable
/// * `user_agent` - User agent string if applicable
/// * `username` - Associated username if available
/// * `url` - Related URL
/// * `destination_domain_name` - Target domain name
/// * `status` - Current notification status
/// * `observation_time` - Time of observation
/// * `source_feed` - Origin threat feed identifier
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NCTNS {
    pub uuid: Uuid,
    pub source_time: String,
    pub time: DateTime<Utc>,
    pub ip: String,
    pub reverse_dns: String,
    pub domain_name: String,
    pub asn: String,
    pub as_name: String,
    pub category: String,
    pub type_: String,
    pub malware_family: String,
    pub vulnerability: String,
    pub tag: String,
    pub source_name: String,
    pub comment: String,
    pub description: String,
    pub description_url: String,
    pub destination_ip: String,
    pub destination_port: i32,
    pub port: i32,
    pub protocol: String,
    pub transport_protocol: String,
    pub http_request: String,
    pub user_agent: String,
    pub username: String,
    pub url: String,
    pub destination_domain_name: String,
    pub status: String,
    pub observation_time: DateTime<Utc>,
    pub source_feed: String,
}

/// Database Row Conversion Implementation
///
/// Enables automatic conversion from database query results to NCTNS instances.
/// Maps column indices to struct fields based on the standard query structure.
///
/// # Column Mapping
/// 0. uuid
/// 1. source_time
/// 2. time
/// ... (continues for all fields in order)
impl From<Row> for NCTNS {
    fn from(row: Row) -> Self {
        Self {
            uuid: row.get(0),
            source_time: row.get(1),
            time: row.get(2),
            ip: row.get(3),
            reverse_dns: row.get(4),
            domain_name: row.get(5),
            asn: row.get(6),
            as_name: row.get(7),
            category: row.get(8),
            type_: row.get(9),
            malware_family: row.get(10),
            vulnerability: row.get(11),
            tag: row.get(12),
            source_name: row.get(13),
            comment: row.get(14),
            description: row.get(15),
            description_url: row.get(16),
            destination_ip: row.get(17),
            destination_port: row.get(18),
            port: row.get(19),
            protocol: row.get(20),
            transport_protocol: row.get(21),
            http_request: row.get(22),
            user_agent: row.get(23),
            username: row.get(24),
            url: row.get(25),
            destination_domain_name: row.get(26),
            status: row.get(27),
            observation_time: row.get(28),
            source_feed: row.get(29),
        }
    }
}

/// NCTNS Database Operations Implementation
///
/// Provides methods for interacting with the NCTNS database table.
/// Supports querying and managing notification records.
impl NCTNS {
    /// Retrieves all NCTNS records from the database.
    ///
    /// # Type Parameters
    /// * `C` - Database client type implementing GenericClient
    ///
    /// # Arguments
    /// * `client` - Database connection client
    ///
    /// # Returns
    /// * `Result<Vec<NCTNS>, Error>` - List of all notifications or database error
    ///
    /// # Database Schema
    /// Requires table 'nctns' with all fields matching struct definition
    pub async fn all<C: GenericClient + Sync>(client: &C) -> Result<Vec<NCTNS>, Error> {
        let stmt = client
            .prepare("SELECT uuid, source_time, time, ip, reverse_dns, domain_name, asn, as_name, category, type, malware_family, vulnerability, tag, source_name, comment, description, description_url, destination_ip, destination_port, port, protocol, transport_protocol, http_request, user_agent, username, url, destination_domain_name, status, observation_time, source_feed FROM nctns")
            .await?;

        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(NCTNS::from).collect())
    }
}
