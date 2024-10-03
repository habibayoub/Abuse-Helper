use actix_web::{dev::ServiceRequest, Error};
use chrono::{Duration, Utc};
use deadpool_postgres::Pool;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

pub fn create_jwt(
    user_id: &str,
    role: &str,
    token_type: TokenType,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = match token_type {
        TokenType::Access => Utc::now() + Duration::hours(1),
        TokenType::Refresh => Utc::now() + Duration::days(7),
    };

    let claims = Claims {
        sub: user_id.to_owned(),
        role: role.to_owned(),
        exp: expiration.timestamp() as usize,
        token_type,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub async fn check_auth(req: &ServiceRequest, pool: &Pool) -> Result<Claims, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid authorization header"))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(actix_web::error::ErrorUnauthorized(
            "Invalid authorization header",
        ));
    }

    let token = &auth_str[7..];

    if is_token_blacklisted(pool, token).await? {
        return Err(actix_web::error::ErrorUnauthorized(
            "Token has been invalidated",
        ));
    }

    match verify_jwt(token) {
        Ok(claims) => {
            if claims.token_type != TokenType::Access {
                return Err(actix_web::error::ErrorUnauthorized("Invalid token type"));
            }
            Ok(claims)
        }
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                Err(actix_web::error::ErrorUnauthorized("Token has expired"))
            }
            _ => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
        },
    }
}

pub fn is_authorized(claims: &Claims, required_role: &str) -> bool {
    claims.role == required_role || claims.role == "admin"
}

pub async fn invalidate_token(pool: &Pool, token: &str) -> Result<(), Error> {
    let client = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    client
        .execute(
            "INSERT INTO blacklisted_tokens (token, created_at) VALUES ($1, NOW())",
            &[&token],
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?;

    Ok(())
}

async fn is_token_blacklisted(pool: &Pool, token: &str) -> Result<bool, Error> {
    let client = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let result = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM blacklisted_tokens WHERE token = $1)",
            &[&token],
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?;

    Ok(result.get(0))
}
