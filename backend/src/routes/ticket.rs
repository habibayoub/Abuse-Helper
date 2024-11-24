use crate::models::ticket::{CreateTicketRequest, Ticket, TicketStatus, TicketType};
use actix_web::{get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Create a new ticket
///
/// Returns a 201 Created on success with the ticket ID
/// Returns a 400 Bad Request if validation fails
/// Returns a 500 Internal Server Error if saving fails
#[post("/create")]
pub async fn create_ticket(
    pool: web::Data<Pool>,
    ticket_req: web::Json<CreateTicketRequest>,
) -> HttpResponse {
    let ticket_data = ticket_req.into_inner();

    let ticket = Ticket::new(
        ticket_data.ticket_type.unwrap_or(TicketType::Other),
        ticket_data.email_id,
        ticket_data.subject,
        ticket_data.description,
        None,
        ticket_data.confidence_score.map(|c| c as f64),
        ticket_data.identified_threats,
        ticket_data.extracted_indicators,
        ticket_data.analysis_summary,
    );

    if let Err(e) = ticket.validate() {
        log::warn!("Ticket validation failed: {}", e);
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match ticket.save(&pool).await {
        Ok(id) => {
            log::info!("Created ticket {}", id);
            HttpResponse::Created().json(id)
        }
        Err(e) => {
            log::error!("Failed to create ticket: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// List all tickets
///
/// Returns a 200 OK with the list of tickets
/// Returns a 500 Internal Server Error if fetching fails
#[get("/list")]
pub async fn list_tickets(pool: web::Data<Pool>) -> HttpResponse {
    match Ticket::list_all(&pool).await {
        Ok(tickets) => {
            log::info!("Retrieved {} tickets", tickets.len());
            HttpResponse::Ok().json(tickets)
        }
        Err(e) => {
            log::error!("Failed to list tickets: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Update ticket status
///
/// Returns a 200 OK on success
/// Returns a 404 Not Found if ticket doesn't exist
/// Returns a 500 Internal Server Error if update fails
#[put("/{id}/status")]
pub async fn update_ticket_status(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    status: web::Json<TicketStatus>,
) -> HttpResponse {
    let id = path.into_inner();
    let new_status = status.into_inner();

    match Ticket::find_by_id(&pool, id).await {
        Ok(Some(mut ticket)) => match ticket.update_status(&pool, new_status).await {
            Ok(_) => {
                log::info!("Updated ticket {} status to {:?}", id, new_status);
                HttpResponse::Ok().finish()
            }
            Err(e) => {
                log::error!("Failed to update ticket {} status: {}", id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        Ok(None) => {
            log::warn!("Ticket {} not found", id);
            HttpResponse::NotFound().json("Ticket not found")
        }
        Err(e) => {
            log::error!("Failed to find ticket {}: {}", id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
