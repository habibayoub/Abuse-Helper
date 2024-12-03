use actix_web::{dev::ServiceRequest, Error};
use chrono::{Duration, Utc};
use deadpool_postgres::Pool;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::models::auth::{Claims, TokenType};
use crate::models::user::User;

/// Creates a new JWT token for a user
///
/// # Arguments
/// * `user_uuid` - User's unique identifier
/// * `role` - User's role (e.g., "user", "admin")
/// * `token_type` - Type of token (Access or Refresh)
///
/// # Returns
/// * `Result<String, jsonwebtoken::errors::Error>` - JWT token or error
///
/// # Token Expiration
/// - Access tokens: 1 hour
/// - Refresh tokens: 7 days
pub fn create_jwt(
    user_uuid: &Uuid,
    role: &str,
    token_type: TokenType,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Set the expiration date based on the token type
    let expiration = match token_type {
        TokenType::Access => Utc::now() + Duration::hours(1),
        TokenType::Refresh => Utc::now() + Duration::days(7),
    };

    // Create the claims for the token
    let claims = Claims {
        sub: user_uuid.clone(),
        role: role.to_owned(),
        exp: expiration.timestamp() as usize,
        token_type,
    };

    // Get the secret from the environment
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Encode the claims into a JWT token
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verifies a JWT token's validity
///
/// # Arguments
/// * `token` - JWT token string
///
/// # Returns
/// * `Result<Claims, jsonwebtoken::errors::Error>` - Token claims or error
///
/// # Validation
/// - Signature verification
/// - Expiration check
/// - Token structure validation
pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // Get the secret from the environment
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Decode the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Checks authentication for incoming requests
///
/// # Arguments
/// * `req` - Service request
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Claims, Error>` - Valid claims or error
///
/// # Validation Steps
/// 1. Checks for Authorization header
/// 2. Validates Bearer token format
/// 3. Checks token blacklist
/// 4. Verifies token validity
/// 5. Validates token type
pub async fn check_auth(req: &ServiceRequest, pool: &Pool) -> Result<Claims, Error> {
    // Log the path being checked
    log::info!("Checking auth for path: {}", req.path());

    // Check for the Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        log::info!("Authorization header found: {:?}", auth_header);

        // Convert the header to a string
        let auth_str = auth_header
            .to_str()
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid authorization header"))?;

        // Check if the header starts with "Bearer "
        if !auth_str.starts_with("Bearer ") {
            return Err(actix_web::error::ErrorUnauthorized(
                "Invalid authorization header",
            ));
        }

        // Extract the token from the header
        let token = &auth_str[7..];

        // Check if the token is blacklisted
        if is_token_blacklisted(pool, token).await? {
            return Err(actix_web::error::ErrorUnauthorized(
                "Token has been invalidated",
            ));
        }

        // Verify the token
        match verify_jwt(token) {
            Ok(claims) => {
                // Check if the token type is valid
                if claims.token_type != TokenType::Access {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token type"));
                }
                Ok(claims)
            }
            // Handle the error
            Err(e) => match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(actix_web::error::ErrorUnauthorized("Token has expired"))
                }
                _ => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
            },
        }
    } else {
        // Log the error
        log::info!("No Authorization header found");
        // Return an unauthorized error
        Err(actix_web::error::ErrorUnauthorized(
            "No authorization header",
        ))
    }
}

/// Checks if a user has required role authorization
///
/// # Arguments
/// * `claims` - User's token claims
/// * `required_role` - Role required for access
///
/// # Returns
/// * `bool` - True if authorized
///
/// # Authorization Rules
/// - Admin role has access to everything
/// - Exact role match required otherwise
pub fn is_authorized(claims: &Claims, required_role: &str) -> bool {
    // Check if the user has the required role or is an admin
    claims.role == required_role || claims.role == "admin"
}

/// Invalidates a JWT token by adding it to blacklist
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `token` - Token to invalidate
///
/// # Returns
/// * `Result<(), Error>` - Success or error
///
/// # Database Operation
/// Inserts token into blacklisted_tokens table
pub async fn invalidate_token(pool: &Pool, token: &str) -> Result<(), Error> {
    // Get a client from the pool
    let client = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    // Insert the token into the blacklisted_tokens table
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
    // Get a client from the pool
    let client = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    // Check if the token exists in the blacklisted_tokens table
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

/// Exchanges a Keycloak token for user information
///
/// # Arguments
/// * `keycloak_token` - Valid Keycloak token
///
/// # Returns
/// * `Result<User, Error>` - User information or error
///
/// # Environment Variables Required
/// - KEYCLOAK_URL
/// - KEYCLOAK_REALM
/// - KEYCLOAK_CLIENT_ID
/// - KEYCLOAK_CLIENT_SECRET
///
/// # Token Validation
/// 1. Validates token with Keycloak
/// 2. Extracts user information
/// 3. Maps Keycloak roles to system roles
pub async fn exchange_keycloak_token(keycloak_token: &str) -> Result<User, Error> {
    // Get the Keycloak URL from the environment
    let keycloak_url = std::env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    // Get the Keycloak realm from the environment
    let realm = std::env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set");
    // Get the Keycloak client ID from the environment
    let client_id = std::env::var("KEYCLOAK_CLIENT_ID").expect("KEYCLOAK_CLIENT_ID must be set");
    // Get the Keycloak client secret from the environment
    let client_secret =
        std::env::var("KEYCLOAK_CLIENT_SECRET").expect("KEYCLOAK_CLIENT_SECRET must be set");

    // Create a new HTTP client
    let client = Client::new();

    // Send a POST request to the Keycloak introspection endpoint
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

    // Parse the response as JSON
    let token_info: Value = response.json().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to parse Keycloak response: {}",
            e
        ))
    })?;

    // Check if the token is active
    if !token_info["active"].as_bool().unwrap_or(false) {
        return Err(actix_web::error::ErrorUnauthorized(
            "Invalid Keycloak token",
        ));
    }

    // Extract the email from the token
    let email = token_info["email"].as_str().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Missing email in Keycloak token")
    })?;

    // Extract the name from the token
    let name = token_info["name"].as_str().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Missing name in Keycloak token")
    })?;

    // Extract the role from the token
    let role = token_info["realm_access"]["roles"]
        .as_array()
        .and_then(|roles| {
            roles
                .iter()
                .find(|&r| r.as_str() == Some("user") || r.as_str() == Some("admin"))
        })
        .and_then(|role| role.as_str())
        .unwrap_or("user");

    // Create a new user
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
