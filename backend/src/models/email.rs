use lettre::message::Mailbox;

/// Struct representing an email in the database.
#[derive(serde::Deserialize)]
pub struct Email {
    pub recipient: Mailbox,
    pub subject: String,
    pub body: String,
    // TODO: vec of file attachments to send
}
