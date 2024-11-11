use crate::{
    llm::{analyze_threat, ThreatAnalysis},
    models::ticket::{CreateTicketRequest, Ticket, TicketStatus, TicketType},
};
use actix_web::{get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

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

    let ticket_data = ticket_req.into_inner();

    // Analyze content with AI if analysis data not provided
    let (
        ticket_type,
        confidence_score,
        identified_threats,
        extracted_indicators,
        analysis_summary,
        ip_address,
    ) = if ticket_data.ticket_type.is_none() {
        let content = format!(
            "Subject: {}\nContent: {}",
            ticket_data.subject, ticket_data.description
        );
        match analyze_threat(&content).await {
            Ok(analysis) => {
                let ip = analysis
                    .extracted_indicators
                    .iter()
                    .find(|indicator| indicator.contains('.'))
                    .cloned();
                (
                    analysis.threat_type,
                    Some(analysis.confidence_score),
                    Some(analysis.identified_threats),
                    Some(analysis.extracted_indicators),
                    Some(analysis.summary),
                    ip,
                )
            }
            Err(e) => {
                log::error!("Failed to analyze ticket content: {}", e);
                (TicketType::Other, None, None, None, None, None)
            }
        }
    } else {
        (
            ticket_data.ticket_type.unwrap(),
            ticket_data.confidence_score,
            ticket_data.identified_threats,
            ticket_data.extracted_indicators,
            ticket_data.analysis_summary,
            None,
        )
    };

    let ticket_id = Uuid::new_v4();

    let stmt = match client
        .prepare(
            "INSERT INTO tickets (
                id, ticket_type, ip_address, email_id, subject, description,
                confidence_score, identified_threats, extracted_indicators, analysis_summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
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
                &confidence_score,
                &identified_threats,
                &extracted_indicators,
                &analysis_summary,
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
