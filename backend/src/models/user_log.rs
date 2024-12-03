use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User activity log entry.
///
/// Represents a single user action or system interaction event.
/// Provides comprehensive tracking of user activities for audit
/// and compliance purposes.
///
/// # Fields
/// * `uuid` - Unique identifier for the log entry
/// * `user_uuid` - Identifier of the user who performed the action
/// * `action` - Description of the performed action
/// * `route` - API route or system path accessed
/// * `timestamp` - When the action occurred (automatically set)
///
/// # Usage
/// ```rust
/// // Example log creation (pseudo-code):
/// UserLog::create(pool, user_id, "login", "/api/auth/login").await?;
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct UserLog {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub action: String,
    pub route: String,
    pub timestamp: DateTime<Utc>,
}

impl UserLog {
    /// Creates a new user activity log entry.
    ///
    /// Records a user action in the database with automatic timestamp
    /// and UUID generation.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_uuid` - ID of the user performing the action
    /// * `action` - Description of the performed action
    /// * `route` - System route or API endpoint accessed
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error
    ///
    /// # Database Schema
    /// Requires table 'user_logs' with columns:
    /// - uuid (UUID, PRIMARY KEY)
    /// - user_uuid (UUID, FOREIGN KEY)
    /// - action (TEXT)
    /// - route (TEXT)
    /// - timestamp (TIMESTAMP WITH TIME ZONE, DEFAULT NOW())
    pub async fn create(
        pool: &Pool,
        user_uuid: Uuid,
        action: &str,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        let uuid = Uuid::new_v4();
        client
            .execute(
                "INSERT INTO user_logs (uuid, user_uuid, action, route) VALUES ($1, $2, $3, $4)",
                &[&uuid, &user_uuid, &action, &route],
            )
            .await?;
        Ok(())
    }
}
