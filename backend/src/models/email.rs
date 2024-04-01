use lettre::message::Mailbox;

#[derive(serde::Deserialize)]
pub struct Email {
    pub recipient: Mailbox,
    pub subject: String,
    pub body: String,
    // attachment
}
