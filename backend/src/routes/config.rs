use crate::middleware::Auth;
use crate::routes;
use actix_web::{get, web, HttpResponse, Scope};

/// GET /status endpoint to check if the server is alive
#[get("status")]
async fn api_status() -> HttpResponse {
    // Return a 200 OK response with a simple message
    HttpResponse::Ok().json("alive")
}

pub fn configure_routes() -> Scope {
    web::scope("")
        .service(api_status)
        .service(
            web::scope("/auth")
                .service(routes::auth::login)
                .service(routes::auth::refresh)
                .service(routes::auth::exchange_token),
        )
        .service(
            web::scope("")
                .wrap(Auth::new())
                .service(web::scope("/auth").service(routes::auth::logout))
                .service(
                    web::scope("/customer")
                        .wrap(Auth::new().role("admin"))
                        .service(routes::customer::list)
                        .service(routes::customer::find),
                )
                .service(
                    web::scope("/nctns")
                        .wrap(Auth::new().role("user"))
                        .service(routes::nctns::list),
                )
                .service(
                    web::scope("/util")
                        .wrap(Auth::new().role("user"))
                        .service(routes::util::whois),
                )
                .service(
                    web::scope("/email")
                        .wrap(Auth::new().role("admin"))
                        .service(routes::email::send),
                ),
        )
}
