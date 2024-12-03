use crate::auth::{
    create_jwt, exchange_keycloak_token, invalidate_token, is_authorized, verify_jwt,
};
use crate::models::auth::{
    CreateUserRequest, ExchangeTokenRequest, LoginForm, RefreshRequest, TokenResponse, TokenType,
    UserResponse,
};
use crate::models::user::User;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// User authentication endpoint.
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/login \
///   -H "Content-Type: application/json" \
///   -d '{"email": "user@example.com", "password": "secure_password123"}'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "access_token": "eyJhbGciOiJIUzI1NiIs...",
///   "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
///   "user": {
///     "uuid": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "name": "John Doe",
///     "role": "user"
///   }
/// }
/// ```
#[post("/login")]
pub async fn login(pool: web::Data<Pool>, form: web::Json<LoginForm>) -> HttpResponse {
    // Find the user by email
    let user = match User::find_by_email(&pool, &form.email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid email or password"),
    };

    // Verify the password
    if !verify(&form.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json("Invalid email or password");
    }

    // Create the access and refresh tokens
    let access_token = create_jwt(&user.uuid, &user.role, TokenType::Access);
    let refresh_token = create_jwt(&user.uuid, &user.role, TokenType::Refresh);

    // Return the tokens and user information
    match (access_token, refresh_token) {
        (Ok(access_str), Ok(refresh_str)) => {
            let token_response = TokenResponse {
                access_token: access_str,
                refresh_token: Some(refresh_str),
                user: UserResponse {
                    uuid: user.uuid,
                    email: user.email,
                    name: user.name,
                    role: user.role,
                },
            };
            HttpResponse::Ok().json(token_response)
        }
        _ => HttpResponse::InternalServerError().finish(),
    }
}

/// Token refresh endpoint.
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/refresh \
///   -H "Content-Type: application/json" \
///   -d '{"refresh_token": "eyJhbGciOiJIUzI1NiIs..."}'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "access_token": "eyJhbGciOiJIUzI1NiIs...",
///   "refresh_token": null,
///   "user": {
///     "uuid": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "name": "John Doe",
///     "role": "user"
///   }
/// }
/// ```
#[post("/refresh")]
pub async fn refresh(pool: web::Data<Pool>, form: web::Json<RefreshRequest>) -> HttpResponse {
    // Verify the refresh token
    match verify_jwt(&form.refresh_token) {
        Ok(claims) => {
            // Check if the token type is refresh
            if claims.token_type != TokenType::Refresh {
                return HttpResponse::BadRequest().json("Invalid token type");
            }

            // Find the user by UUID
            let user = match User::find_by_uuid(&pool, &claims.sub).await {
                Ok(user) => user,
                Err(_) => return HttpResponse::InternalServerError().json("Failed to fetch user"),
            };

            // Create the new access token
            let new_access_token = create_jwt(&user.uuid, &user.role, TokenType::Access);

            // Return the new access token and user information
            match new_access_token {
                Ok(access_token) => {
                    let token_response = TokenResponse {
                        access_token,
                        refresh_token: None, // We don't issue a new refresh token here
                        user: UserResponse {
                            uuid: user.uuid,
                            email: user.email,
                            name: user.name,
                            role: user.role,
                        },
                    };
                    HttpResponse::Ok().json(token_response)
                }
                Err(_) => HttpResponse::InternalServerError().json("Failed to create access token"),
            }
        }
        Err(_) => HttpResponse::Unauthorized().json("Invalid refresh token"),
    }
}

/// User logout endpoint.
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/logout \
///   -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
/// ```
///
/// # Example Response
/// ```json
/// "Logged out successfully"
/// ```
#[post("/logout")]
pub async fn logout(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    // Check if the authorization header is present
    if let Some(auth_header) = req.headers().get("Authorization") {
        // Convert the header to a string
        if let Ok(auth_str) = auth_header.to_str() {
            // Check if the string starts with "Bearer "
            if auth_str.starts_with("Bearer ") {
                // Extract the token from the header
                let token = &auth_str[7..];
                // Invalidate the token
                match invalidate_token(&pool, token).await {
                    Ok(_) => return HttpResponse::Ok().json("Logged out successfully"),
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
    HttpResponse::BadRequest().json("No token provided")
}

/// Keycloak token exchange endpoint.
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/exchange \
///   -H "Content-Type: application/json" \
///   -d '{"keycloak_token": "eyJhbGciOiJSUzI1NiIs..."}'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "access_token": "eyJhbGciOiJIUzI1NiIs...",
///   "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
///   "user": {
///     "uuid": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "name": "John Doe",
///     "role": "user"
///   }
/// }
/// ```
#[post("/exchange")]
pub async fn exchange_token(
    exchange_request: web::Json<ExchangeTokenRequest>,
    pool: web::Data<Pool>,
) -> impl Responder {
    // Exchange the Keycloak token for a user
    match exchange_keycloak_token(&exchange_request.keycloak_token).await {
        Ok(keycloak_user) => {
            // Find the user by email
            let user = match User::find_by_email(&pool, &keycloak_user.email).await {
                // Return the user if found
                Ok(user) => user,
                Err(_) => {
                    // User doesn't exist, create a new user
                    match User::create(
                        &pool,
                        Uuid::new_v4(),
                        keycloak_user.email,
                        keycloak_user.name,
                        "".to_string(), // We don't need a password for Keycloak users
                        keycloak_user.role,
                    )
                    .await
                    {
                        // Return the user if created
                        Ok(user) => user,
                        // Return an error if the user creation failed
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to create user"
                            }))
                        }
                    }
                }
            };

            // Create the access and refresh tokens
            match create_jwt(&user.uuid, &user.role, TokenType::Access) {
                // Create the access token
                Ok(access_token) => match create_jwt(&user.uuid, &user.role, TokenType::Refresh) {
                    // Create the refresh token
                    Ok(refresh_token) => {
                        // Return the tokens and user information
                        let token_response = TokenResponse {
                            access_token,
                            refresh_token: Some(refresh_token),
                            user: UserResponse {
                                uuid: user.uuid,
                                email: user.email,
                                name: user.name,
                                role: user.role,
                            },
                        };
                        HttpResponse::Ok().json(token_response)
                    }
                    // Return an error if the refresh token creation failed
                    Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create refresh token"
                    })),
                },
                // Return an error if the access token creation failed
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to create access token"
                })),
            }
        }
        // Return an error if the Keycloak token exchange failed
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// User creation endpoint (admin only).
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/create_user \
///   -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
///   -H "Content-Type: application/json" \
///   -d '{
///     "email": "newuser@example.com",
///     "name": "New User",
///     "password": "secure_password123",
///     "role": "user"
///   }'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "uuid": "123e4567-e89b-12d3-a456-426614174000",
///   "email": "newuser@example.com",
///   "name": "New User",
///   "role": "user"
/// }
/// ```
///
/// # Error Response Example
/// ```json
/// {
///   "error": "Insufficient permissions"
/// }
/// ```
#[post("/create_user")]
pub async fn create_user(
    req: HttpRequest,
    pool: web::Data<Pool>,
    form: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Check if the requester is authenticated and is an admin
    let auth_header = req.headers().get("Authorization");

    // Check if the authorization header is present
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().json("No authorization header");
    }

    // Extract the token from the header
    let token = auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .replace("Bearer ", "");

    // Verify the token
    let claims = match verify_jwt(&token) {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid token"),
    };

    // Check if the user is authorized
    if !is_authorized(&claims, "admin") {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    // Hash the password
    let password_hash = match hash(&form.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to hash password"),
    };

    // Create the new user
    let new_user = match User::create(
        &pool,
        Uuid::new_v4(),
        form.email.clone(),
        form.name.clone(),
        password_hash,
        form.role.clone(),
    )
    .await
    {
        // Return the user if created
        Ok(user) => user,
        // Return an error if the user creation failed
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(format!("Failed to create user: {}", e))
        }
    };

    // Return the user if created
    HttpResponse::Ok().json(UserResponse {
        uuid: new_user.uuid,
        email: new_user.email,
        name: new_user.name,
        role: new_user.role,
    })
}
