use actix_web::{dev::ServiceRequest, Error};
use chrono::{Duration, Utc};
use deadpool_postgres::Pool;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::models::auth::{Claims, TokenType};
use crate::models::user::User;

pub fn create_jwt(
    user_uuid: &Uuid,
    role: &str,
    token_type: TokenType,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = match token_type {
        TokenType::Access => Utc::now() + Duration::hours(1),
        TokenType::Refresh => Utc::now() + Duration::days(7),
    };

    let claims = Claims {
        sub: user_uuid.clone(),
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
    log::debug!("Checking auth for path: {}", req.path());
    if let Some(auth_header) = req.headers().get("Authorization") {
        log::debug!("Authorization header found: {:?}", auth_header);
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
    } else {
        log::debug!("No Authorization header found");
        Err(actix_web::error::ErrorUnauthorized(
            "No authorization header",
        ))
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

pub async fn exchange_keycloak_token(keycloak_token: &str) -> Result<User, Error> {
    let keycloak_url = std::env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    let realm = std::env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set");
    let client_id = std::env::var("KEYCLOAK_CLIENT_ID").expect("KEYCLOAK_CLIENT_ID must be set");
    let client_secret =
        std::env::var("KEYCLOAK_CLIENT_SECRET").expect("KEYCLOAK_CLIENT_SECRET must be set");

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/realms/{}/protocol/openid-connect/token/introspect",
            keycloak_url, realm
        ))
        .form(&[
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("token", &keycloak_token.to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Keycloak request failed: {}", e))
        })?;

    let token_info: Value = response.json().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to parse Keycloak response: {}",
            e
        ))
    })?;

    if !token_info["active"].as_bool().unwrap_or(false) {
        return Err(actix_web::error::ErrorUnauthorized(
            "Invalid Keycloak token",
        ));
    }

    let email = token_info["email"].as_str().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Missing email in Keycloak token")
    })?;

    let name = token_info["name"].as_str().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Missing name in Keycloak token")
    })?;

    // Needs to be an array, fix in the future
    let role = token_info["realm_access"]["roles"]
        .as_array()
        .and_then(|roles| {
            roles
                .iter()
                .find(|&r| r.as_str() == Some("user") || r.as_str() == Some("admin"))
        })
        .and_then(|role| role.as_str())
        .unwrap_or("user");

    let user = User {
        uuid: Uuid::new_v4(),
        email: email.to_owned(),
        name: name.to_owned(),
        password_hash: "".to_string(),
        role: role.to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(user)
}
