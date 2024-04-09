use chrono::NaiveDateTime;
use tokio_postgres::{Error, GenericClient, Row};

/// Struct representing a row in the nctns table.
#[derive(Debug, serde::Serialize)]
pub struct NCTNS {
    pub uuid: String,
    pub source_time: String,
    pub time: NaiveDateTime,
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
    pub observation_time: NaiveDateTime,
    pub source_feed: String,
}

/// Implement the `From<Row>` trait for `NCTNS`.
/// This allows us to convert a `tokio_postgres::Row` into a `NCTNS`.
impl From<Row> for NCTNS {
    fn from(row: Row) -> Self {
        Self {
            uuid: row.get(0),
            source_time: row.get(1),
            time: row.get::<usize, NaiveDateTime>(2),
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
            observation_time: row.get::<usize, NaiveDateTime>(28),
            source_feed: row.get(29),
        }
    }
}

/// Implement the `NCTNS` struct.
/// Methods on the `NCTNS` struct allow us to interact with the database.
impl NCTNS {
    pub async fn all<C: GenericClient + Sync>(client: &C) -> Result<Vec<NCTNS>, Error> {
        let stmt = client
            .prepare("SELECT uuid, source_time, time, ip, reverse_dns, domain_name, asn, as_name, category, type, malware_family, vulnerability, tag, source_name, comment, description, description_url, destination_ip, destination_port, port, protocol, transport_protocol, http_request, user_agent, username, url, destination_domain_name, status, observation_time, source_feed FROM nctns")
            .await?;

        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(NCTNS::from).collect())
    }
}
