use tokio_postgres::{Error, GenericClient, Row};

#[derive(serde::Deserialize)]
pub struct EmailForm {
    pub email: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Customer {
    pub id: i32,
    pub email: String,
}

impl From<Row> for Customer {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(0),
            email: row.get(1),
        }
    }
}

impl Customer {
    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Customer>, Error> {
        let stmt = client.prepare("SELECT id, email FROM customers").await?;
        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(Customer::from).collect())
    }
    pub async fn find_by_email<C: GenericClient>(
        client: &C,
        email: &str,
    ) -> Result<Customer, Error> {
        let stmt = client
            .prepare("SELECT id, email FROM customers WHERE email = $1")
            .await?;
        let row = client.query_one(&stmt, &[&email]).await?;

        Ok(Customer::from(row))
    }
}
