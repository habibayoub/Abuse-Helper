use actix_web::{get, post, web, HttpResponse};
use deadpool_postgres::Pool;

use crate::models::customer::{Customer, LookUpForm};

/// GET /customer/list endpoint to list all customers
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

    // Fetch all customers from the database
    match Customer::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch customers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch customers");
        }
    }
}

/// POST /customer/find endpoint to find a customer by email, ip, or id
#[post("/find")]
pub async fn find(pool: web::Data<Pool>, form: web::Json<LookUpForm>) -> HttpResponse {
    // Get a connection from the pool
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    // Find the customer by email, ip, or id
    match (form.email.clone(), form.ip.clone(), form.id.clone()) {
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
        // If id is provided, find by id
        (_, _, Some(id)) => match Customer::find_by_id(&**client, id).await {
            Ok(customer) => HttpResponse::Ok().json(customer),
            Err(_) => HttpResponse::InternalServerError().json("customer not found"),
        },
        // If none of the above are provided, return an error
        _ => HttpResponse::InternalServerError()
            .json("please enter a valid lookup value [email | ip]"),
    }
}
