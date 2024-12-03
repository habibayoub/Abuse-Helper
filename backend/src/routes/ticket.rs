use crate::models::requests::{AddEmailRequest, CreateTicketRequest, CreateTicketResponse};
use crate::models::ticket::{SearchOptions, Ticket, TicketError, TicketStatus, TicketType};
use actix_web::{delete, get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Create a new ticket
///
/// # Endpoint
/// POST /tickets/create
///
/// # Request Body
/// ```json
/// {
///   "ticket_type": "INCIDENT",
///   "subject": "Security Incident",
///   "description": "Detailed description",
///   "email_ids": ["uuid1", "uuid2"],
///   "confidence_score": 0.85,
///   "identified_threats": ["malware", "phishing"],
///   "extracted_indicators": ["ip:192.168.1.1"],
///   "analysis_summary": "Threat analysis details"
/// }
/// ```
///
/// # Returns
/// - 201: Ticket created successfully
/// - 200: Ticket created with some email link failures
/// - 400: Validation error
/// - 500: Database error
#[post("/create")]
pub async fn create_ticket(
    pool: web::Data<Pool>,
    ticket_req: web::Json<CreateTicketRequest>,
) -> HttpResponse {
    // Extract the ticket data
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
        // Save the ticket
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
///
/// # Endpoint
/// POST /tickets/{id}/emails
///
/// # Path Parameters
/// - id: Ticket UUID
///
/// # Request Body
/// ```json
/// {
///   "email_id": "123e4567-e89b-12d3-a456-426614174000"
/// }
/// ```
#[post("/{id}/emails")]
pub async fn add_email_to_ticket(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    email_req: web::Json<AddEmailRequest>,
) -> HttpResponse {
    // Extract the ticket and email IDs
    let ticket_id = path.into_inner();
    let email_id = email_req.email_id;

    // Find the ticket
    match Ticket::find_by_id(&pool, ticket_id).await {
        // If the ticket exists, add the email
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
///
/// # Endpoint
/// DELETE /tickets/{id}/emails/{email_id}
///
/// # Path Parameters
/// - id: Ticket UUID
/// - email_id: Email UUID to remove
#[delete("/{id}/emails/{email_id}")]
pub async fn remove_email_from_ticket(
    pool: web::Data<Pool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    // Extract the ticket and email IDs
    let (ticket_id, email_id) = path.into_inner();

    // Find the ticket
    match Ticket::find_by_id(&pool, ticket_id).await {
        // If the ticket exists, remove the email
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

/// Get all emails associated with a ticket
///
/// # Endpoint
/// GET /tickets/{id}/emails
///
/// # Path Parameters
/// - id: Ticket UUID
///
/// # Returns
/// Array of email UUIDs as strings
#[get("/{id}/emails")]
pub async fn get_ticket_emails(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the ticket ID
    let ticket_id = path.into_inner();

    // Find the ticket
    match Ticket::find_by_id(&pool, ticket_id).await {
        // If the ticket exists, get the emails
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

/// List all tickets in the system
///
/// # Endpoint
/// GET /tickets/list
///
/// # Returns
/// Array of ticket objects with full details
#[get("/list")]
pub async fn list_tickets(pool: web::Data<Pool>) -> HttpResponse {
    // List all tickets
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
/// # Endpoint
/// PUT /tickets/{id}/status
///
/// # Path Parameters
/// - id: Ticket UUID
///
/// # Request Body
/// String representing new status (e.g., "OPEN", "CLOSED")
#[put("/{id}/status")]
pub async fn update_ticket_status(
    pool: web::Data<Pool>,
    path: web::Path<Uuid>,
    status: web::Json<String>,
) -> HttpResponse {
    // Extract the ticket ID and new status
    let id = path.into_inner();
    let new_status = TicketStatus::from(status.into_inner());

    // Find the ticket
    match Ticket::find_by_id(&pool, id).await {
        // If the ticket exists, update the status
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
///
/// # Endpoint
/// GET /tickets/{id}
///
/// # Path Parameters
/// - id: Ticket UUID
///
/// # Returns
/// Complete ticket object with all fields
#[get("/{id}")]
pub async fn get_ticket(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the ticket ID
    let id = path.into_inner();

    // Find the ticket
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

/// Search tickets based on various criteria
///
/// # Endpoint
/// GET /tickets/search
///
/// # Query Parameters
/// - q: Search query string
/// - status: Filter by status
/// - type: Filter by ticket type
/// - from: Pagination offset
/// - size: Page size
/// - sort: Sort field
/// - order: Sort direction
///
/// # Returns
/// Paginated search results with metadata
#[get("/search")]
pub async fn search_tickets(query: web::Query<SearchOptions>) -> HttpResponse {
    // Extract the search options
    let search_options = query.into_inner();
    log::debug!("Search request: {:?}", search_options);

    // Search for tickets
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
