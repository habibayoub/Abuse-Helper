use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims structure containing user authentication and authorization data.
///
/// Represents the decoded content of a JWT token, containing essential
/// information about the user's identity, permissions, and token validity.
///
/// # Fields
/// * `sub` - Subject identifier (user UUID)
/// * `role` - User's role for authorization
/// * `exp` - Token expiration timestamp
/// * `token_type` - Distinguishes between access and refresh tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub exp: usize,
    pub token_type: TokenType,
}

/// Defines the types of JWT tokens supported by the system.
///
/// Used to differentiate between access tokens (short-lived, for API access)
/// and refresh tokens (long-lived, for obtaining new access tokens).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TokenType {
    /// Short-lived token for API access
    Access,
    /// Long-lived token for refreshing access tokens
    Refresh,
}

/// User login request data structure.
///
/// Contains credentials required for user authentication.
/// Used in login endpoints to process authentication requests.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    /// User's email address
    pub email: String,
    /// User's password (plain text - will be hashed)
    pub password: String,
}

/// Token refresh request structure.
///
/// Used to request a new access token using a valid refresh token.
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    /// Valid refresh token for obtaining new access token
    pub refresh_token: String,
}

/// External token exchange request structure.
///
/// Used for exchanging third-party (Keycloak) tokens for system tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeTokenRequest {
    /// Valid Keycloak token to be exchanged
    pub keycloak_token: String,
}

/// Authentication response containing tokens and user data.
///
/// Returned after successful authentication or token refresh operations.
/// Contains both authentication tokens and user profile information.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// JWT access token for API authorization
    pub access_token: String,
    /// Optional refresh token for obtaining new access tokens
    pub refresh_token: Option<String>,
    /// User profile information
    pub user: UserResponse,
}

/// User profile response structure.
///
/// Contains user information returned in authentication responses
/// and user profile endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    /// Unique identifier for the user
    pub uuid: Uuid,
    /// User's email address
    pub email: String,
    /// User's display name
    pub name: String,
    /// User's role for authorization
    pub role: String,
}

/// New user creation request structure.
///
/// Contains required information for creating a new user account.
/// Used in registration and user management endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// Email address for the new user
    pub email: String,
    /// Display name for the new user
    pub name: String,
    /// Initial password (will be hashed)
    pub password: String,
    /// Initial role assignment
    pub role: String,
}
