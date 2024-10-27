use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize)]
pub struct UserLog {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub action: String,
    pub route: String,
    pub timestamp: DateTime<Utc>,
}

impl UserLog {
    pub async fn create(
        pool: &Pool,
        user_uuid: Uuid,
        action: &str,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        client
            .execute(
                "INSERT INTO user_logs (user_uuid, action, route) VALUES ($1, $2, $3)",
                &[&user_uuid, &action, &route],
            )
            .await?;
        Ok(())
    }
}
