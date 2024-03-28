#[derive(serde::Deserialize)]
pub struct Email {
    pub subject: String,
    pub recipient: String,
    pub body: String,
}