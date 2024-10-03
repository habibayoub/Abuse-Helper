use actix_web::{post, web, HttpRequest, HttpResponse};
use bcrypt::verify;
use deadpool_postgres::Pool;

use crate::auth::{create_jwt, invalidate_token, verify_jwt, TokenType};
use crate::models::auth::{LoginForm, RefreshRequest, TokenResponse};
use crate::models::user::User;

#[post("/login")]
pub async fn login(pool: web::Data<Pool>, form: web::Json<LoginForm>) -> HttpResponse {
    let user = match User::find_by_email(&pool, &form.email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid email or password"),
    };

    if !verify(&form.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json("Invalid email or password");
    }

    let access_token = create_jwt(&user.id.to_string(), &user.role, TokenType::Access);
    let refresh_token = create_jwt(&user.id.to_string(), &user.role, TokenType::Refresh);

    match (access_token, refresh_token) {
        (Ok(access_str), Ok(refresh_str)) => HttpResponse::Ok().json(TokenResponse {
            access_token: access_str,
            refresh_token: refresh_str,
        }),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/refresh")]
pub async fn refresh(form: web::Json<RefreshRequest>) -> HttpResponse {
    match verify_jwt(&form.refresh_token) {
        Ok(claims) => {
            if claims.token_type != TokenType::Refresh {
                return HttpResponse::BadRequest().json("Invalid token type");
            }
            let new_access_token = create_jwt(&claims.sub, &claims.role, TokenType::Access);
            match new_access_token {
                Ok(token) => HttpResponse::Ok().json(token),
                Err(_) => HttpResponse::InternalServerError().finish(),
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
