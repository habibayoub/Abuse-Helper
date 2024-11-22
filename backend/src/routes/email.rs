use crate::models::email::{Email, EmailError, OutgoingEmail};
use actix_web::{get, post, web, HttpResponse};
use deadpool_postgres::Pool;

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

/// Lists all unanalyzed emails and processes them
///
/// Returns a 200 OK with the list of emails on success
/// Returns a 500 Internal Server Error if fetching or processing fails
#[get("/list")]
pub async fn list_emails(pool: web::Data<Pool>) -> HttpResponse {
    match Email::fetch_unanalyzed(&pool).await {
        Ok(mut emails) => {
            let results = Email::process_batch(&pool, &mut emails).await;

            // Simplify the partition
            let (success, failure): (Vec<_>, Vec<_>) = results.iter().partition(|r| r.is_ok());

            log::info!(
                "Processed {} emails: {} successful, {} failed",
                results.len(),
                success.len(),
                failure.len()
            );

            // Log any errors that occurred during processing
            for (i, result) in results.iter().enumerate() {
                if let Err(e) = result {
                    log::error!("Failed to process email {}: {}", emails[i].id, e);
                }
            }

            HttpResponse::Ok().json(emails)
        }
        Err(e) => {
            log::error!("Failed to fetch emails: {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
