use actix_web::{get, web, HttpResponse};
use deadpool_postgres::Pool;

use crate::models::nctns::NCTNS;

// GET /nctns/list endpoint to list all NCTNS records
#[get("/list")]
pub async fn list(pool: web::Data<Pool>) -> HttpResponse {
    // Get a connection from the pool
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    // Fetch all NCTNS records from the database
    match NCTNS::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch NCTNS records: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch NCTNS records");
        }
    }
}
