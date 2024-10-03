use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub async fn find_by_email(
        pool: &Pool,
        email: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        let row = client
            .query_one(
                "SELECT id, email, password_hash, role FROM users WHERE email = $1",
                &[&email],
            )
            .await?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            password_hash: row.get(2),
            role: row.get(3),
        })
    }
}
