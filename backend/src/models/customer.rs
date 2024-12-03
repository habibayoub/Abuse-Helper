use chrono::{DateTime, Utc};
use tokio_postgres::{Error, GenericClient, Row};
use uuid::Uuid;

/// Customer lookup form for flexible search operations.
///
/// Provides multiple optional search criteria for customer lookup operations.
/// All fields are optional to support partial matching and different search strategies.
///
/// # Fields
/// * `email` - Customer's email address for lookup
/// * `ip` - IP address associated with customer
/// * `uuid` - Unique identifier for direct customer lookup
#[derive(serde::Deserialize, serde::Serialize)]
pub struct LookUpForm {
    pub email: Option<String>,
    pub ip: Option<String>,
    pub uuid: Option<Uuid>,
}

/// Customer entity representation.
///
/// Represents a customer record in the database with all associated information.
/// Supports serialization for API responses and deserialization for database operations.
///
/// # Fields
/// * `uuid` - Unique identifier for the customer
/// * `email` - Primary contact email
/// * `first_name` - Optional first name
/// * `last_name` - Optional last name
/// * `ip` - Last known IP address
/// * `created_at` - Account creation timestamp
/// * `updated_at` - Last modification timestamp
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Customer {
    pub uuid: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row conversion implementation for Customer.
///
/// Enables automatic conversion from database rows to Customer instances,
/// mapping column names to struct fields.
impl From<Row> for Customer {
    fn from(row: Row) -> Self {
        Self {
            uuid: row.get("uuid"),
            email: row.get("email"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            ip: row.get("ip"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Customer database operations implementation.
///
/// Provides methods for interacting with customer records in the database.
/// Supports CRUD operations and various lookup strategies.
impl Customer {
    /// Retrieves all customers from the database.
    ///
    /// # Arguments
    /// * `client` - Database client for executing queries
    ///
    /// # Returns
    /// * `Result<Vec<Customer>, Error>` - List of all customers or database error
    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Customer>, Error> {
        let stmt = client
            .prepare("SELECT uuid, email, first_name, last_name, ip, created_at, updated_at FROM customers")
            .await?;
        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(Customer::from).collect())
    }

    /// Finds a customer by their email address.
    ///
    /// # Arguments
    /// * `client` - Database client for executing queries
    /// * `email` - Email address to search for
    ///
    /// # Returns
    /// * `Result<Customer, Error>` - Matching customer or database error
    pub async fn find_by_email<C: GenericClient>(
        client: &C,
        email: &str,
    ) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT uuid, email, first_name, last_name, ip, created_at, updated_at FROM customers WHERE email = $1")
            .await?;
        let row = client.query_one(&stmt, &[&email]).await?;

        Ok(Customer::from(row))
    }

    /// Finds a customer by their IP address.
    ///
    /// # Arguments
    /// * `client` - Database client for executing queries
    /// * `ip` - IP address to search for
    ///
    /// # Returns
    /// * `Result<Customer, Error>` - Matching customer or database error
    pub async fn find_by_ip<C: GenericClient>(client: &C, ip: &str) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT uuid, email, first_name, last_name, ip, created_at, updated_at FROM customers WHERE ip = $1")
            .await?;
        let row = client.query_one(&stmt, &[&ip]).await?;

        Ok(Customer::from(row))
    }

    /// Finds a customer by their UUID.
    ///
    /// # Arguments
    /// * `client` - Database client for executing queries
    /// * `uuid` - UUID to search for
    ///
    /// # Returns
    /// * `Result<Customer, Error>` - Matching customer or database error
    pub async fn find_by_uuid<C: GenericClient>(client: &C, uuid: Uuid) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT uuid, email, first_name, last_name, ip, created_at, updated_at FROM customers WHERE uuid = $1")
            .await?;
        let row = client.query_one(&stmt, &[&uuid]).await?;

        Ok(Customer::from(row))
    }
}
