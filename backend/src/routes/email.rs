use actix_web::{get, web, HttpResponse};
use lettre::{message::header::ContentType, AsyncTransport};

use crate::models::email::Email;

async fn send_email(email: Email) -> Result<String, String> {
    // SMTP server configuration
    let smtp_server =
        std::env::var("SMTP_SERVER").expect("please ensure SMTP_SERVER is set in the environment");
    let smtp_username = std::env::var("SMTP_USERNAME")
        .expect("please ensure SMTP_USERNAME is set in the environment");
    let smtp_password = std::env::var("SMTP_PASSWORD")
        .expect("please ensure SMTP_PASSWORD is set in the environment");

    let recipient = email.recipient.email.clone();

    // Prepare the email response
    let email_payload = lettre::Message::builder()
        .from(smtp_username.parse().unwrap())
        .to(email.recipient)
        .subject(&email.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(email.body.to_string())
        .unwrap();

    let creds =
        lettre::transport::smtp::authentication::Credentials::new(smtp_username, smtp_password);

    let mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor> =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

    match mailer.send(email_payload).await {
        Ok(_) => Ok(format!(
            "successfully sent {} to {}",
            email.subject, recipient
        )),
        Err(e) => Err(format!("Could not send email: {:#?}", e)),
    }
}

#[get("/send")]
pub async fn send(email: web::Json<Email>) -> HttpResponse {
    match send_email(email.into_inner()).await {
        Ok(result) => HttpResponse::Created().json(result),
        Err(e) => HttpResponse::InternalServerError().json(format!("{}", e)),
    }
}
