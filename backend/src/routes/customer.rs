use actix_web::{get, post, web, HttpResponse};
use deadpool_postgres::Pool;

use crate::models::auth::Claims;
use crate::models::customer::{Customer, LookUpForm};

/// Lists all customers in the system.
///
/// # Endpoint
/// GET /customer/list
///
/// # Authorization
/// Requires authenticated user with admin role
///
/// # Example Request
/// ```bash
/// curl -X GET http://api.example.com/customer/list \
///   -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
/// ```
///
/// # Example Response
/// ```json
/// [
///   {
///     "uuid": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "customer@example.com",
///     "first_name": "Example",
///     "last_name": "Customer",
///     "ip": "192.168.1.1",
///     "created_at": "2021-01-01T00:00:00Z",
///     "updated_at": "2021-01-01T00:00:00Z"
///   }
/// ]
/// ```
///
/// # Errors
/// - 500: Database connection failed
/// - 500: Customer fetch failed
/// - 401: Unauthorized
/// - 403: Insufficient permissions
#[get("/list")]
pub async fn list(pool: web::Data<Pool>, claims: web::ReqData<Claims>) -> HttpResponse {
    log::info!("User {} is listing customers", claims.sub);

    // Get a connection from the pool
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::info!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    // Fetch all customers from the database
    match Customer::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::info!("unable to fetch customers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch customers");
        }
    }
}

/// Finds a customer by email, IP address, or UUID.
///
/// # Endpoint
/// POST /customer/find
///
/// # Request Body
/// ```json
/// {
///   "email": "customer@example.com",  // optional
///   "ip": "192.168.1.1",             // optional
///   "uuid": "123e4567-..."           // optional
/// }
/// ```
///
/// # Search Priority
/// 1. Email (if provided)
/// 2. IP address (if provided and email not present)
/// 3. UUID (if provided and neither email nor IP present)
///
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/customer/find \
///   -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
///   -H "Content-Type: application/json" \
///   -d '{"email": "customer@example.com"}'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "uuid": "123e4567-e89b-12d3-a456-426614174000",
///   "email": "customer@example.com",
///   "first_name": "Example",
///   "last_name": "Customer",
///   "ip": "192.168.1.1",
///   "created_at": "2021-01-01T00:00:00Z",
///   "updated_at": "2021-01-01T00:00:00Z"
/// }
/// ```
///
/// # Errors
/// - 500: Database connection failed
/// - 500: Customer not found
/// - 500: Invalid lookup parameters
/// - 401: Unauthorized
/// - 403: Insufficient permissions
#[post("/find")]
pub async fn find(
    pool: web::Data<Pool>,
    form: web::Json<LookUpForm>,
    claims: web::ReqData<Claims>,
) -> HttpResponse {
    log::info!("User {} is finding a customer", claims.sub);

    // Get a connection from the pool
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::info!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    // Find the customer by email, ip, or id
    match (form.email.clone(), form.ip.clone(), form.uuid.clone()) {
        // If email is provided, find by email
        (Some(email), _, _) => {
            match Customer::find_by_email(&**client, &email).await {
                Ok(customer) => return HttpResponse::Ok().json(customer),
                Err(_) => return HttpResponse::InternalServerError().json("customer not found"),
            };
        }
        // If ip is provided, find by ip
        (_, Some(ip), _) => match Customer::find_by_ip(&**client, &ip).await {
            Ok(customer) => return HttpResponse::Ok().json(customer),
            Err(_) => return HttpResponse::InternalServerError().json("customer not found"),
        },
        // If uuid is provided, find by uuid
        (_, _, Some(uuid)) => match Customer::find_by_uuid(&**client, uuid).await {
            Ok(customer) => HttpResponse::Ok().json(customer),
            Err(_) => HttpResponse::InternalServerError().json("customer not found"),
        },
        // If none of the above are provided, return an error
        _ => HttpResponse::InternalServerError()
            .json("please enter a valid lookup value [email | ip]"),
    }
}
