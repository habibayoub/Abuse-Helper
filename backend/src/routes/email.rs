use crate::models::email::Email;
use actix_web::{get, web, HttpResponse};
use lettre::{message::header::ContentType, AsyncTransport};
use native_tls::TlsConnector;

/// Function to send an email using the SMTP server
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

    // Authenticate with the SMTP server
    let creds =
        lettre::transport::smtp::authentication::Credentials::new(smtp_username, smtp_password);

    // Build the mailer
    let mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor> =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

    // Send the email
    match mailer.send(email_payload).await {
        Ok(_) => Ok(format!(
            "successfully sent {} to {}",
            email.subject, recipient
        )),
        Err(e) => Err(format!("Could not send email: {:#?}", e)),
    }
}

/// GET /email/send endpoint to send an email using the SMTP server
#[get("/send")]
pub async fn send(email: web::Json<Email>) -> HttpResponse {
    match send_email(email.into_inner()).await {
        Ok(result) => HttpResponse::Created().json(result),
        Err(e) => HttpResponse::InternalServerError().json(format!("{}", e)),
    }
}

/// Function to poll the IMAP server for new emails
pub async fn poll() {
    // IMAP server configuration
    let imap_server =
        std::env::var("IMAP_SERVER").expect("please ensure IMAP_SERVER is set in the environment");
    let imap_username = std::env::var("IMAP_USERNAME")
        .expect("please ensure SMTP_USERNAME is set in the environment");
    let imap_password = std::env::var("IMAP_PASSWORD")
        .expect("please ensure SMTP_PASSWORD is set in the environment");

    // Connect to the IMAP server
    let tls = TlsConnector::builder().build().unwrap();
    let client = imap::connect((imap_server.clone(), 993), imap_server, &tls).unwrap();
    let mut imap_session = client.login(imap_username, imap_password).unwrap();

    // Select the INBOX
    imap_session.select("INBOX").unwrap();

    // Fetch all new emails
    let messages = imap_session.fetch("1:*", "RFC822").unwrap();
    let messages = messages.iter().collect::<Vec<_>>();
    if !messages.is_empty() {
        println!("You have {} new emails", messages.len());
    }

    // TODO:
    // document IP for email
    // dummy email bodies
    // seen this ip at this timestamp and port, etc
    // put that info into the DB, and then search the database for that IP instance
    // if correlations, shoot an email to that user
    // enter in custom email body and atatachments
    // log email sent to DB
    // how many times per individual, track that in stats

    // Logout from the IMAP server
    imap_session.logout().unwrap();
}
