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
                        .service(routes::email::send)
                        .service(routes::email::list_emails)
                        .service(routes::email::process_emails)
                        .service(routes::email::delete_email)
                        .service(routes::email::mark_analyzed)
                        .service(routes::email::get_email_tickets)
                        .service(routes::email::link_to_ticket)
                        .service(routes::email::unlink_from_ticket)
                        .service(routes::email::force_delete_email)
                        .service(routes::email::search_emails),
                )
                .service(
                    web::scope("/tickets")
                        .wrap(Auth::new().role("user"))
                        .service(routes::ticket::create_ticket)
                        .service(routes::ticket::list_tickets)
                        .service(routes::ticket::update_ticket_status)
                        .service(routes::ticket::add_email_to_ticket)
                        .service(routes::ticket::remove_email_from_ticket)
                        .service(routes::ticket::get_ticket_emails)
                        .service(routes::ticket::get_ticket),
                ),
        )
}
