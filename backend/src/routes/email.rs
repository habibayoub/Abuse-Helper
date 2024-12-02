use crate::models::email::{Email, EmailError, OutgoingEmail, SearchOptions};
use actix_web::{delete, get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Sends an outgoing email and saves it to the database
///
/// Returns a 201 Created on success with the send result
/// Returns a 400 Bad Request if validation fails
/// Returns a 500 Internal Server Error if sending or saving fails
#[post("/send")]
pub async fn send(pool: web::Data<Pool>, email: web::Json<OutgoingEmail>) -> HttpResponse {
    let email_data = email.into_inner();

    if let Err(e) = email_data.validate() {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match email_data.send().await {
        Ok(result) => match email_data.save(&pool).await {
            Ok(_) => HttpResponse::Created().json(result),
            Err(e) => {
                log::error!("Failed to save sent email: {}", e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
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
/// Returns a 200 OK with the list of emails on success
/// Returns a 500 Internal Server Error if fetching fails
#[get("/list")]
pub async fn list_emails(pool: web::Data<Pool>) -> HttpResponse {
    match Email::fetch_all(&pool).await {
        Ok(emails) => {
            // Find unanalyzed email IDs
            let unanalyzed_ids: Vec<Uuid> = emails
                .iter()
                .filter(|email| !email.analyzed)
                .map(|email| email.id.clone())
                .collect();

            if !unanalyzed_ids.is_empty() {
                log::info!(
                    "Starting background processing of {} unanalyzed emails",
                    unanalyzed_ids.len()
                );

                // Clone what we need for the background task
                let pool = pool.clone();

                // Spawn background task
                actix_web::rt::spawn(async move {
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

            HttpResponse::Ok().json(emails)
        }
        Err(e) => {
            log::error!("Failed to fetch emails: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

/// Process a batch of emails by their IDs
#[post("/process")]
pub async fn process_emails(pool: web::Data<Pool>, ids: web::Json<Vec<Uuid>>) -> HttpResponse {
    match Email::process_batch_by_ids(&pool, &ids).await {
        Ok(results) => {
            let (success, failure): (Vec<_>, Vec<_>) = results.iter().partition(|r| r.is_ok());

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
#[delete("/{id}")]
pub async fn delete_email(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let email_id = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(email) => match email.delete(&pool).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(EmailError::Validation(msg)) => {
                log::warn!("Cannot delete email {}: {}", email_id, msg);
                HttpResponse::BadRequest().json(msg)
            }
            Err(e) => {
                log::error!("Failed to delete email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
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
#[put("/{id}/analyze")]
pub async fn mark_analyzed(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let email_id = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(mut email) => match email.mark_as_analyzed(&pool).await {
            Ok(_) => HttpResponse::Ok().json("Email marked as analyzed"),
            Err(e) => {
                log::error!("Failed to mark email {} as analyzed: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
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
#[get("/{id}/tickets")]
pub async fn get_email_tickets(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let email_id = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(email) => match email.get_tickets(&pool).await {
            Ok(tickets) => HttpResponse::Ok().json(tickets),
            Err(e) => {
                log::error!("Failed to get tickets for email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
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
#[post("/{id}/tickets/{ticket_id}")]
pub async fn link_to_ticket(pool: web::Data<Pool>, path: web::Path<(Uuid, Uuid)>) -> HttpResponse {
    let (email_id, ticket_id) = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(email) => match email.link_ticket(&pool, ticket_id).await {
            Ok(_) => {
                log::info!("Linked email {} to ticket {}", email_id, ticket_id);
                HttpResponse::Ok().finish()
            }
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
#[delete("/{id}/tickets/{ticket_id}")]
pub async fn unlink_from_ticket(
    pool: web::Data<Pool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (email_id, ticket_id) = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(email) => match email.unlink_ticket(&pool, ticket_id).await {
            Ok(_) => {
                log::info!("Unlinked email {} from ticket {}", email_id, ticket_id);
                HttpResponse::NoContent().finish()
            }
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
#[delete("/{id}/force")]
pub async fn force_delete_email(pool: web::Data<Pool>, path: web::Path<Uuid>) -> HttpResponse {
    let email_id = path.into_inner();

    match Email::fetch_by_id(&pool, &email_id).await {
        Ok(email) => match email.force_delete(&pool).await {
            Ok(_) => {
                log::info!(
                    "Force deleted email {} and removed all ticket associations",
                    email_id
                );
                HttpResponse::NoContent().finish()
            }
            Err(e) => {
                log::error!("Failed to force delete email {}: {}", email_id, e);
                HttpResponse::InternalServerError().json(e.to_string())
            }
        },
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

/// Search emails
#[get("/search")]
pub async fn search_emails(query: web::Query<SearchOptions>) -> HttpResponse {
    let search_options = query.into_inner();
    log::debug!("Search request: {:?}", search_options);

    match Email::search(search_options).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Failed to search emails: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
