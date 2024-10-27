use crate::models::ticket::{Ticket, TicketStatus, TicketType};
use actix_web::{get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use regex::Regex;
use uuid::Uuid;

/// Function to extract IP addresses from text
fn extract_ips(text: &str) -> Vec<String> {
    let ip_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    ip_regex
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Function to determine ticket type from content
fn determine_ticket_type(subject: &str, body: &str) -> TicketType {
    let text = format!("{} {}", subject, body).to_lowercase();

    if text.contains("malware") || text.contains("virus") || text.contains("trojan") {
        TicketType::Malware
    } else if text.contains("phishing") || text.contains("credential") || text.contains("login") {
        TicketType::Phishing
    } else if text.contains("scam") || text.contains("fraud") {
        TicketType::Scam
    } else if text.contains("spam") {
        TicketType::Spam
    } else {
        TicketType::Other
    }
}

#[derive(serde::Deserialize)]
pub struct CreateTicketRequest {
    pub email_id: String,
    pub subject: String,
    pub description: String,
    pub ticket_type: Option<TicketType>,
}

/// POST /tickets endpoint to create a new ticket
#[post("/tickets")]
pub async fn create_ticket(
    pool: web::Data<Pool>,
    ticket_req: web::Json<CreateTicketRequest>,
) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let ticket_data = ticket_req.into_inner(); // Extract the data first
    let ip_addresses = extract_ips(&ticket_data.description);
    let ip_address = ip_addresses.first().map(|s| s.to_string());

    let ticket_type = ticket_data
        .ticket_type
        .unwrap_or_else(|| determine_ticket_type(&ticket_data.subject, &ticket_data.description));

    let ticket_id = Uuid::new_v4();

    let stmt = match client
        .prepare(
            "INSERT INTO tickets (id, ticket_type, ip_address, email_id, subject, description) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    match client
        .query_one(
            &stmt,
            &[
                &ticket_id,
                &ticket_type.to_string(),
                &ip_address,
                &ticket_data.email_id,
                &ticket_data.subject,
                &ticket_data.description,
            ],
        )
        .await
    {
        Ok(_) => HttpResponse::Created().json(ticket_id),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/tickets")]
pub async fn list_tickets(pool: web::Data<Pool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    match client
        .query("SELECT * FROM tickets ORDER BY created_at DESC", &[])
        .await
    {
        Ok(rows) => {
            let tickets: Vec<Ticket> = rows.into_iter().map(Ticket::from).collect();
            HttpResponse::Ok().json(tickets)
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[put("/tickets/{id}/status")]
pub async fn update_ticket_status(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    status: web::Json<TicketStatus>,
) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let id = path.into_inner();
    let status = status.into_inner();

    match client
        .execute(
            "UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2",
            &[&status.to_string(), &id],
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
