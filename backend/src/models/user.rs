use actix_web::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

/// User account representation.
///
/// Represents a user account in the system with associated metadata
/// and authentication information.
///
/// # Fields
/// * `uuid` - Unique identifier for the user
/// * `email` - User's email address (unique)
/// * `name` - Display name
/// * `password_hash` - Securely hashed password
/// * `role` - User's system role
/// * `created_at` - Account creation timestamp
/// * `updated_at` - Last modification timestamp
///
/// # Security Notes
/// - Passwords are never stored in plain text
/// - Email addresses must be unique
/// - Roles determine access permissions
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub uuid: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Finds a user by their email address.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `email` - Email address to search for
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Found user or error
    ///
    /// # Database Query
    /// Searches the users table for an exact email match
    pub async fn find_by_email(pool: &Pool, email: &str) -> Result<Self, Error> {
        // Get a database connection from the pool
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

        // Execute the query to find the user by email
        let row = client
            .query_one(
                "SELECT uuid, email, name, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
                &[&email],
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error finding user: {}", e))
            })?;

        // Convert the row to a User object
        Ok(User {
            uuid: row.get::<_, Uuid>("uuid"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Creates a new user account.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `uuid` - Unique identifier for the new user
    /// * `email` - User's email address
    /// * `name` - Display name
    /// * `password_hash` - Pre-hashed password
    /// * `role` - Initial user role
    ///
    /// # Returns
    /// * `Result<User, Error>` - Created user or error
    ///
    /// # Database Operations
    /// - Inserts new user record
    /// - Returns complete user object with timestamps
    /// - Enforces unique email constraint
    pub async fn create(
        pool: &Pool,
        uuid: Uuid,
        email: String,
        name: String,
        password_hash: String,
        role: String,
    ) -> Result<User, Error> {
        // Get a database connection from the pool
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

        // Execute the query to insert the new user
        let row = client
            .query_one(
                "INSERT INTO users (uuid, email, name, password_hash, role) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING uuid, email, name, password_hash, role, created_at, updated_at",
                &[
                    &uuid as &(dyn ToSql + Sync),
                    &email,
                    &name,
                    &password_hash,
                    &role,
                ],
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error creating user: {}", e))
            })?;

        // Convert the row to a User object
        Ok(User {
            uuid: row.get::<_, Uuid>("uuid"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Finds a user by their UUID.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `uuid` - UUID to search for
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Found user or error
    ///
    /// # Database Query
    /// Searches the users table for an exact UUID match
    ///
    /// # Error Handling
    /// - Returns error if user not found
    /// - Handles database connection failures
    pub async fn find_by_uuid(pool: &Pool, uuid: &Uuid) -> Result<Self, Error> {
        // Get a database connection from the pool
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

        // Execute the query to find the user by UUID
        let row = client
            .query_one(
                "SELECT uuid, email, name, password_hash, role, created_at, updated_at FROM users WHERE uuid = $1",
                &[&uuid as &(dyn ToSql + Sync)],
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error finding user: {}", e))
            })?;

        Ok(User {
            uuid: row.get::<_, Uuid>("uuid"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
