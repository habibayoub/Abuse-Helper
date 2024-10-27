use actix_web::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

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
    pub async fn find_by_email(pool: &Pool, email: &str) -> Result<Self, Error> {
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

        let row = client
            .query_one(
                "SELECT uuid, email, name, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
                &[&email],
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

    pub async fn create(
        pool: &Pool,
        uuid: Uuid,
        email: String,
        name: String,
        password_hash: String,
        role: String,
    ) -> Result<User, Error> {
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

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

    pub async fn find_by_uuid(pool: &Pool, uuid: &Uuid) -> Result<Self, Error> {
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

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
