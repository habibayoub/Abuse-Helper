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

#[post("/login")]
pub async fn login(pool: web::Data<Pool>, form: web::Json<LoginForm>) -> HttpResponse {
    let user = match User::find_by_email(&pool, &form.email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid email or password: find"),
    };

    if !verify(&form.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json("Invalid email or password: verify");
    }

    let access_token = create_jwt(&user.uuid, &user.role, TokenType::Access);
    let refresh_token = create_jwt(&user.uuid, &user.role, TokenType::Refresh);

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

#[post("/refresh")]
pub async fn refresh(pool: web::Data<Pool>, form: web::Json<RefreshRequest>) -> HttpResponse {
    match verify_jwt(&form.refresh_token) {
        Ok(claims) => {
            if claims.token_type != TokenType::Refresh {
                return HttpResponse::BadRequest().json("Invalid token type");
            }

            let user = match User::find_by_uuid(&pool, &claims.sub).await {
                Ok(user) => user,
                Err(_) => return HttpResponse::InternalServerError().json("Failed to fetch user"),
            };

            let new_access_token = create_jwt(&user.uuid, &user.role, TokenType::Access);
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

#[post("/logout")]
pub async fn logout(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                match invalidate_token(&pool, token).await {
                    Ok(_) => return HttpResponse::Ok().json("Logged out successfully"),
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
    HttpResponse::BadRequest().json("No token provided")
}

#[post("/exchange")]
pub async fn exchange_token(
    exchange_request: web::Json<ExchangeTokenRequest>,
    pool: web::Data<Pool>,
) -> impl Responder {
    match exchange_keycloak_token(&exchange_request.keycloak_token).await {
        Ok(keycloak_user) => {
            let user = match User::find_by_email(&pool, &keycloak_user.email).await {
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
                        Ok(user) => user,
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to create user"
                            }))
                        }
                    }
                }
            };

            match create_jwt(&user.uuid, &user.role, TokenType::Access) {
                Ok(access_token) => match create_jwt(&user.uuid, &user.role, TokenType::Refresh) {
                    Ok(refresh_token) => {
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
                    Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create refresh token"
                    })),
                },
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to create access token"
                })),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[post("/create_user")]
pub async fn create_user(
    req: HttpRequest,
    pool: web::Data<Pool>,
    form: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Check if the requester is authenticated and is an admin
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().json("No authorization header");
    }

    let token = auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .replace("Bearer ", "");
    let claims = match verify_jwt(&token) {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid token"),
    };

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
        Ok(user) => user,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(format!("Failed to create user: {}", e))
        }
    };

    HttpResponse::Ok().json(UserResponse {
        uuid: new_user.uuid,
        email: new_user.email,
        name: new_user.name,
        role: new_user.role,
    })
}
