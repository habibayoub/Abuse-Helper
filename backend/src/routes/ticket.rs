use crate::models::requests::{AddEmailRequest, CreateTicketRequest, CreateTicketResponse};
use crate::models::ticket::{SearchOptions, Ticket, TicketError, TicketStatus, TicketType};
use actix_web::{delete, get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Create a new ticket
#[post("/create")]
pub async fn create_ticket(
    pool: web::Data<Pool>,
    ticket_req: web::Json<CreateTicketRequest>,
) -> HttpResponse {
    let ticket_data = ticket_req.into_inner();

    // Validate the request
    if let Err(e) = ticket_data.validate() {
        log::warn!("Ticket request validation failed: {}", e);
        return HttpResponse::BadRequest().json(e.to_string());
    }

    // Create the ticket
    let ticket = Ticket::new(
        ticket_data.ticket_type.unwrap_or(TicketType::Other),
        ticket_data.subject,
        ticket_data.description,
        None,
        ticket_data.confidence_score,
        ticket_data.identified_threats,
        ticket_data.extracted_indicators,
        ticket_data.analysis_summary,
    );

    // If no emails to link, use simple save
    if ticket_data.email_ids.is_empty() {
        match ticket.save(&pool).await {
            Ok(ticket_id) => {
                log::info!("Created standalone ticket {}", ticket_id);
                return HttpResponse::Created().json(CreateTicketResponse {
                    ticket_id,
                    linked_emails: vec![],
                    failed_emails: vec![],
                });
            }
            Err(e) => {
                log::error!("Failed to create ticket: {}", e);
                return HttpResponse::InternalServerError().json(e.to_string());
            }
        }
    }

    // If we have emails to link, use transaction
    let mut client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json("Database connection error");
        }
    };

    // Start a transaction
    let tx = match client.transaction().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json("Transaction error");
        }
    };

    // Save the ticket within the transaction
    let ticket_id = match ticket.save_with_client(&tx).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to create ticket: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    };

    let mut linked_emails = Vec::new();
    let mut failed_emails = Vec::new();
    let email_count = ticket_data.email_ids.len();

    // Try to link each email within the transaction
    for email_id in ticket_data.email_ids {
        match ticket.add_email_with_client(&tx, &email_id).await {
            Ok(_) => {
                linked_emails.push(email_id.to_string());
            }
            Err(e) => {
                log::error!("Failed to link email {} to ticket: {}", email_id, e);
                failed_emails.push((email_id.to_string(), e.to_string()));
            }
        }
    }

    // If all email links failed and we had emails to link, rollback
    if linked_emails.is_empty() && email_count > 0 {
        let _ = tx.rollback().await;
        return HttpResponse::BadRequest().json("No valid emails could be linked to the ticket");
    }

    // Otherwise commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json("Failed to commit transaction");
    }

    let response = CreateTicketResponse {
        ticket_id,
        linked_emails: linked_emails.clone(),
        failed_emails: failed_emails.clone(),
    };

    if failed_emails.is_empty() {
        log::info!(
            "Created ticket {} with all emails linked successfully",
            ticket_id
        );
        HttpResponse::Created().json(response)
    } else {
        log::warn!("Created ticket {} but some email links failed", ticket_id);
        HttpResponse::Ok().json(response)
    }
}

/// Add an email to a ticket
#[post("/{id}/emails")]
pub async fn add_email_to_ticket(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    email_req: web::Json<AddEmailRequest>,
) -> HttpResponse {
    let ticket_id = path.into_inner();
    let email_id = email_req.email_id;

    match Ticket::find_by_id(&pool, ticket_id).await {
        Ok(Some(ticket)) => match ticket.add_email(&pool, &email_id).await {
            Ok(_) => {
                log::info!("Added email {} to ticket {}", email_id, ticket_id);
                HttpResponse::Ok().finish()
            }
            Err(TicketError::Validation(msg)) => {
                log::warn!("Validation error adding email to ticket: {}", msg);
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!("Failed to add email to ticket: {}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        Ok(None) => {
            log::warn!("Ticket {} not found", ticket_id);
            HttpResponse::NotFound().json("Ticket not found")
        }
        Err(e) => {
            log::error!("Failed to find ticket {}: {}", ticket_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Remove an email from a ticket
#[delete("/{id}/emails/{email_id}")]
pub async fn remove_email_from_ticket(
    pool: web::Data<Pool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (ticket_id, email_id) = path.into_inner();

    match Ticket::find_by_id(&pool, ticket_id).await {
        Ok(Some(ticket)) => match ticket.remove_email(&pool, &email_id).await {
            Ok(_) => {
                log::info!("Removed email {} from ticket {}", email_id, ticket_id);
                HttpResponse::Ok().finish()
            }
            Err(TicketError::Validation(msg)) => {
                log::warn!("Validation error removing email from ticket: {}", msg);
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!("Failed to remove email from ticket: {}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        Ok(None) => {
            log::warn!("Ticket {} not found", ticket_id);
            HttpResponse::NotFound().json("Ticket not found")
        }
        Err(e) => {
            log::error!("Failed to find ticket {}: {}", ticket_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Get all emails for a ticket
#[get("/{id}/emails")]
pub async fn get_ticket_emails(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let ticket_id = path.into_inner();

    match Ticket::find_by_id(&pool, ticket_id).await {
        Ok(Some(ticket)) => match ticket.get_emails(&pool).await {
            Ok(emails) => {
                let email_strings: Vec<String> = emails.iter().map(|id| id.to_string()).collect();
                log::info!("Retrieved {} emails for ticket {}", emails.len(), ticket_id);
                HttpResponse::Ok().json(email_strings)
            }
            Err(e) => {
                log::error!("Failed to get emails for ticket: {}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        Ok(None) => {
            log::warn!("Ticket {} not found", ticket_id);
            HttpResponse::NotFound().json("Ticket not found")
        }
        Err(e) => {
            log::error!("Failed to find ticket {}: {}", ticket_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// List all tickets
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
#[put("/{id}/status")]
pub async fn update_ticket_status(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    status: web::Json<String>,
) -> HttpResponse {
    let id = path.into_inner();
    let new_status = TicketStatus::from(status.into_inner());

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

/// Get a single ticket by ID
#[get("/{id}")]
pub async fn get_ticket(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();

    match Ticket::find_by_id(&pool, id).await {
        Ok(Some(ticket)) => HttpResponse::Ok().json(ticket),
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

/// Search tickets
#[get("/search")]
pub async fn search_tickets(query: web::Query<SearchOptions>) -> HttpResponse {
    let search_options = query.into_inner();
    log::debug!("Search request: {:?}", search_options);

    match Ticket::search(search_options).await {
        Ok(response) => {
            log::debug!("Search found {} results", response.hits.len());
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to search tickets: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
