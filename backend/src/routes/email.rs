use crate::models::email::{Email, EmailError, OutgoingEmail, SearchOptions};
use actix_web::{delete, get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Sends an outgoing email and saves it to the database
///
/// # Endpoint
/// POST /email/send
///
/// # Request Body
/// ```json
/// {
///   "to": "recipient@example.com",
///   "subject": "Email Subject",
///   "body": "Email content"
/// }
/// ```
///
/// # Returns
/// - 201: Email sent and saved successfully
/// - 400: Validation failed
/// - 500: Sending or saving failed
#[post("/send")]
pub async fn send(pool: web::Data<Pool>, email: web::Json<OutgoingEmail>) -> HttpResponse {
    // Extract the email data from the request body
    let email_data = email.into_inner();

    // Validate the email data
    if let Err(e) = email_data.validate() {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    // Send the email
    match email_data.send().await {
        // Save the sent email to the database
        Ok(result) => match email_data.save(&pool).await {
            Ok(_) => HttpResponse::Created().json(result),
            Err(e) => {
                log::error!("Failed to save sent email: {}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email sending failed
        Err(e) => {
            log::error!("Failed to send email: {}", e);
            match e {
                EmailError::Validation(_) => HttpResponse::BadRequest().json(e.to_string()),
                _ => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
    }
}

/// Lists all emails and processes unanalyzed ones in the background
///
/// # Endpoint
/// GET /email/list
///
/// # Features
/// - Automatic background processing of unanalyzed emails
/// - Batch processing status logging
/// - Error tracking for failed analyses
///
/// # Returns
/// - 200: List of all emails
/// - 500: Database error
#[get("/list")]
pub async fn list_emails(pool: web::Data<Pool>) -> HttpResponse {
    // Fetch all emails from the database
    match Email::fetch_all(&pool).await {
        Ok(emails) => {
            // Find unanalyzed email IDs
            let unanalyzed_ids: Vec<Uuid> = emails
                .iter()
                .filter(|email| !email.analyzed)
                .map(|email| email.id.clone())
                .collect();

            // If there are unanalyzed emails, start background processing
            if !unanalyzed_ids.is_empty() {
                log::info!(
                    "Starting background processing of {} unanalyzed emails",
                    unanalyzed_ids.len()
                );

                // Clone what we need for the background task
                let pool = pool.clone();

                // Spawn background task
                actix_web::rt::spawn(async move {
                    // Process the batch of unanalyzed emails
                    match Email::process_batch_by_ids(&pool, &unanalyzed_ids).await {
                        Ok(results) => {
                            // Log processing results
                            let (success, failure): (Vec<_>, Vec<_>) =
                                results.iter().partition(|r| r.is_ok());

                            log::info!(
                                "Processed {} unanalyzed emails: {} successful, {} failed",
                                results.len(),
                                success.len(),
                                failure.len()
                            );

                            // Log any errors that occurred during processing
                            for (i, result) in results.iter().enumerate() {
                                if let Err(e) = result {
                                    log::error!(
                                        "Failed to process email {}: {}",
                                        unanalyzed_ids[i],
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to process batch: {}", e);
                        }
                    }
                });
            }

            // Return the list of emails
            HttpResponse::Ok().json(emails)
        }
        Err(e) => {
            log::error!("Failed to fetch emails: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Process a batch of emails by their IDs
///
/// # Endpoint
/// POST /email/process
///
/// # Request Body
/// ```json
/// ["uuid1", "uuid2", "uuid3"]
/// ```
///
/// # Returns
/// - 200: Processing results summary
/// - 500: Batch processing failed
#[post("/process")]
pub async fn process_emails(pool: web::Data<Pool>, ids: web::Json<Vec<Uuid>>) -> HttpResponse {
    // Process the batch of emails by their IDs
    match Email::process_batch_by_ids(&pool, &ids).await {
        Ok(results) => {
            // Partition the results into successful and failed ones
            let (success, failure): (Vec<_>, Vec<_>) = results.iter().partition(|r| r.is_ok());

            // Return a success message if all emails were processed successfully
            if failure.is_empty() {
                HttpResponse::Ok().json(format!("Successfully processed {} emails", success.len()))
            } else {
                HttpResponse::Ok().json(format!(
                    "Processed {} emails with {} failures",
                    success.len(),
                    failure.len()
                ))
            }
        }
        Err(e) => {
            log::error!("Failed to process email batch: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Delete an email by ID
///
/// # Endpoint
/// DELETE /email/{id}
///
/// # Parameters
/// - id: Email UUID
///
/// # Returns
/// - 204: Email deleted
/// - 400: Cannot delete (has associations)
/// - 404: Email not found
/// - 500: Deletion failed
#[delete("/{id}")]
pub async fn delete_email(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the email ID from the path
    let email_id = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Delete the email
        Ok(email) => match email.delete(&pool).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            // Return an error if the email deletion failed
            Err(EmailError::Validation(msg)) => {
                log::warn!("Cannot delete email {}: {}", email_id, msg);
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!("Failed to delete email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Mark an email as analyzed
///
/// # Endpoint
/// PUT /email/{id}/analyze
///
/// # Parameters
/// - id: Email UUID
///
/// # Returns
/// - 200: Email marked as analyzed
/// - 404: Email not found
/// - 500: Update failed
#[put("/{id}/analyze")]
pub async fn mark_analyzed(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the email ID from the path
    let email_id = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Mark the email as analyzed
        Ok(mut email) => match email.mark_as_analyzed(&pool).await {
            Ok(_) => HttpResponse::Ok().json("Email marked as analyzed"),
            Err(e) => {
                log::error!("Failed to mark email {} as analyzed: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Get all tickets associated with an email
///
/// # Endpoint
/// GET /email/{id}/tickets
///
/// # Parameters
/// - id: Email UUID
///
/// # Returns
/// - 200: List of associated tickets
/// - 404: Email not found
/// - 500: Fetch failed
#[get("/{id}/tickets")]
pub async fn get_email_tickets(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the email ID from the path
    let email_id = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Get the tickets associated with the email
        Ok(email) => match email.get_tickets(&pool).await {
            Ok(tickets) => HttpResponse::Ok().json(tickets),
            Err(e) => {
                log::error!("Failed to get tickets for email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Link an email to a ticket
///
/// # Endpoint
/// POST /email/{id}/tickets/{ticket_id}
///
/// # Parameters
/// - id: Email UUID
/// - ticket_id: Ticket UUID
///
/// # Returns
/// - 200: Link created
/// - 400: Invalid link request
/// - 404: Email/ticket not found
/// - 500: Link failed
#[post("/{id}/tickets/{ticket_id}")]
pub async fn link_to_ticket(pool: web::Data<Pool>, path: web::Path<(Uuid, Uuid)>) -> HttpResponse {
    // Extract the email ID and ticket ID from the path
    let (email_id, ticket_id) = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Link the email to the ticket
        Ok(email) => match email.link_ticket(&pool, ticket_id).await {
            Ok(_) => {
                log::info!("Linked email {} to ticket {}", email_id, ticket_id);
                HttpResponse::Ok().finish()
            }
            // Return an error if the email linking failed
            Err(EmailError::Validation(msg)) => {
                log::warn!(
                    "Cannot link email {} to ticket {}: {}",
                    email_id,
                    ticket_id,
                    msg
                );
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!(
                    "Failed to link email {} to ticket {}: {}",
                    email_id,
                    ticket_id,
                    e
                );
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Unlink an email from a ticket
///
/// # Endpoint
/// DELETE /email/{id}/tickets/{ticket_id}
///
/// # Parameters
/// - id: Email UUID
/// - ticket_id: Ticket UUID
///
/// # Returns
/// - 204: Link removed
/// - 400: Invalid unlink request
/// - 404: Link not found
/// - 500: Unlink failed
#[delete("/{id}/tickets/{ticket_id}")]
pub async fn unlink_from_ticket(
    pool: web::Data<Pool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    // Extract the email ID and ticket ID from the path
    let (email_id, ticket_id) = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Unlink the email from the ticket
        Ok(email) => match email.unlink_ticket(&pool, ticket_id).await {
            Ok(_) => {
                log::info!("Unlinked email {} from ticket {}", email_id, ticket_id);
                HttpResponse::NoContent().finish()
            }
            // Return an error if the email unlinking failed
            Err(EmailError::Validation(msg)) => {
                log::warn!(
                    "Cannot unlink email {} from ticket {}: {}",
                    email_id,
                    ticket_id,
                    msg
                );
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!(
                    "Failed to unlink email {} from ticket {}: {}",
                    email_id,
                    ticket_id,
                    e
                );
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Force delete an email and remove all ticket associations
///
/// # Endpoint
/// DELETE /email/{id}/force
///
/// # Parameters
/// - id: Email UUID
///
/// # Security Considerations
/// - Irreversible operation
/// - Removes all associations
/// - Should be used with caution
///
/// # Returns
/// - 204: Email and associations deleted
/// - 404: Email not found
/// - 500: Deletion failed
#[delete("/{id}/force")]
pub async fn force_delete_email(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    // Extract the email ID from the path
    let email_id = path.into_inner();

    // Fetch the email by ID
    match Email::fetch_by_id(&pool, &email_id).await {
        // Force delete the email and remove all ticket associations
        Ok(email) => match email.force_delete(&pool).await {
            Ok(_) => {
                log::info!(
                    "Force deleted email {} and removed all ticket associations",
                    email_id
                );
                HttpResponse::NoContent().finish()
            }
            // Return an error if the email force deletion failed
            Err(e) => {
                log::error!("Failed to force delete email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
        // Return an error if the email was not found
        Err(EmailError::Validation(msg)) => {
            log::warn!("Email {} not found", email_id);
            HttpResponse::NotFound().json(msg)
        }
        // Return an error if the email fetch failed
        Err(e) => {
            log::error!("Failed to fetch email {}: {}", email_id, e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Search emails
///
/// # Endpoint
/// GET /email/search
///
/// # Query Parameters
/// - q: Search query string
/// - from: Pagination offset
/// - size: Page size
/// - sort: Sort field
/// - order: Sort direction
///
/// # Returns
/// - 200: Search results
/// - 500: Search failed
#[get("/search")]
pub async fn search_emails(query: web::Query<SearchOptions>) -> HttpResponse {
    // Extract the search options from the query
    let search_options = query.into_inner();

    // Log the search request
    log::debug!("Search request: {:?}", search_options);

    // Search for emails
    match Email::search(search_options).await {
        // Return the search results
        Ok(response) => {
            log::debug!("Search found {} results", response.hits.len());
            HttpResponse::Ok().json(response)
        }
        // Return an error if the search failed
        Err(e) => {
            log::error!("Failed to search emails: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
