use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLog {
    pub id: i32,
    pub user_id: i32,
    pub action: String,
    pub route: String,
    pub timestamp: DateTime<Utc>,
}

impl UserLog {
    pub async fn create(
        pool: &Pool,
        user_id: i32,
        action: &str,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        client
            .execute(
                "INSERT INTO user_logs (user_id, action, route) VALUES ($1, $2, $3)",
                &[&user_id, &action, &route],
            )
            .await?;
        Ok(())
    }
}
