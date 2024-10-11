use actix_web::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
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
                "SELECT id, email, name, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
                &[&email],
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error finding user: {}", e))
            })?;

        Ok(User {
            id: row.get::<_, Uuid>("id"),
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
        id: Uuid,
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
                "INSERT INTO users (id, email, name, password_hash, role) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id, email, name, password_hash, role, created_at, updated_at",
                &[
                    &id as &(dyn ToSql + Sync),
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
            id: row.get::<_, Uuid>("id"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn find_by_id(pool: &Pool, id: &str) -> Result<Self, Error> {
        let client = pool.get().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Error getting db connection: {}",
                e
            ))
        })?;

        let uuid = Uuid::parse_str(id)
            .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid UUID: {}", e)))?;

        let row = client
            .query_one(
                "SELECT id, email, name, password_hash, role, created_at, updated_at FROM users WHERE id = $1",
                &[&uuid as &(dyn ToSql + Sync)],
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error finding user: {}", e))
            })?;

        Ok(User {
            id: row.get::<_, Uuid>("id"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
