use crate::middleware::Auth;
use crate::routes;
use actix_web::{get, web, HttpResponse, Scope};

/// Server health check endpoint
///
/// # Endpoint
/// GET /status
///
/// # Returns
/// - 200 OK with "alive" message
///
/// # Example Request
/// ```bash
/// curl http://api.example.com/status
/// ```
///
/// # Example Response
/// ```json
/// "alive"
/// ```
#[get("status")]
async fn api_status() -> HttpResponse {
    HttpResponse::Ok().json("alive")
}

/// Configures all application routes and their middleware.
///
/// # Route Groups
///
/// ## Public Routes
/// - `GET /status` - Server health check
/// - `POST /auth/login` - User authentication
/// - `POST /auth/refresh` - Token refresh
/// - `POST /auth/exchange` - Token exchange
///
/// ## Protected Routes (Requires Authentication)
/// - `POST /auth/logout` - User logout
///
/// ## Admin Routes
/// - `/customer/*` - Customer management
/// - `/email/*` - Email operations
///
/// ## User Routes
/// - `/nctns/*` - Security notifications
/// - `/util/*` - Utility functions
/// - `/tickets/*` - Ticket management
///
/// # Middleware Configuration
/// - Authentication required for protected routes
/// - Role-based access for admin/user routes
/// - Scoped middleware application
///
/// # Example URL Structure
/// ```text
/// /status                     -> Health check
/// /auth/login                 -> Authentication
/// /customer/list              -> List customers (admin)
/// /email/send                 -> Send email (admin)
/// /tickets/create_ticket      -> Create ticket (user)
/// /nctns/list                -> List notifications (user)
/// ```
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
                        .service(routes::ticket::get_ticket)
                        .service(routes::ticket::search_tickets),
                ),
        )
}
