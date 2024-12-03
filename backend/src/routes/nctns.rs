use crate::models::nctns::NCTNS;
use actix_web::{get, web, HttpResponse};
use deadpool_postgres::Pool;

/// Lists all NCTNS (Network and Cyber Threat Notification System) records
///
/// # Endpoint
/// GET /nctns/list
///
/// # Authorization
/// Requires authenticated user with appropriate permissions
///
/// # Returns
/// ## Success (200 OK)
/// Returns a JSON array of NCTNS records:
/// ```json
/// [
///   {
///     "uuid": "123e4567-e89b-12d3-a456-426614174000",
///     "source_time": "2023-01-01T00:00:00Z",
///     "time": "2023-01-01T00:00:00Z",
///     "ip": "192.168.1.1",
///     "category": "malware",
///     // ... other fields ...
///   }
/// ]
/// ```
///
/// ## Errors
/// - 500: Database connection failed
/// - 500: Query execution failed
#[get("/list")]
pub async fn list(pool: web::Data<Pool>) -> HttpResponse {
    // Get a connection from the pool
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::info!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    // Fetch all NCTNS records from the database
    match NCTNS::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::info!("unable to fetch NCTNS records: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch NCTNS records");
        }
    }
}
