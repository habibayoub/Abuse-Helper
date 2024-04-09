use tokio_postgres::{Error, GenericClient, Row};

/// Form for looking up a customer.
#[derive(serde::Deserialize)]
pub struct LookUpForm {
    pub email: Option<String>,
    pub ip: Option<String>,
    pub id: Option<i32>,
}

/// Struct representing a customer in the database.
#[derive(Debug, serde::Serialize)]
pub struct Customer {
    pub id: i32,
    pub email: String,
    pub ip: String,
}

/// Implement the `From<Row>` trait for `Customer`.
/// This allows us to convert a `tokio_postgres::Row` into a `Customer`.
impl From<Row> for Customer {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(0),
            email: row.get(1),
            ip: row.get(2),
        }
    }
}

/// Implement the `Customer` struct.
/// Methods on the `Customer` struct allow us to interact with the database.
impl Customer {
    /// Get all customers from the database.
    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Customer>, Error> {
        let stmt = client
            .prepare("SELECT id, email, ip FROM customers")
            .await?;
        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(Customer::from).collect())
    }

    /// Find a customer by email.
    pub async fn find_by_email<C: GenericClient>(
        client: &C,
        email: &str,
    ) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT id, email, ip FROM customers WHERE email = $1")
            .await?;
        let row = client.query_one(&stmt, &[&email]).await?;

        Ok(Customer::from(row))
    }

    /// Find a customer by IP address.
    pub async fn find_by_ip<C: GenericClient>(client: &C, ip: &str) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT id, email, ip FROM customers WHERE ip = $1")
            .await?;
        let row = client.query_one(&stmt, &[&ip]).await?;

        Ok(Customer::from(row))
    }

    /// Find a customer by ID.
    pub async fn find_by_id<C: GenericClient>(client: &C, id: i32) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT id, email, ip FROM customers WHERE id = $1")
            .await?;
        let row = client.query_one(&stmt, &[&id]).await?;

        Ok(Customer::from(row))
    }
}
