use actix_web::{get, post, web, HttpResponse};
use deadpool_postgres::Pool;

use crate::models::customer::{Customer, EmailForm};

#[get("/customers")]
pub async fn list_customers(pool: web::Data<Pool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match Customer::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch customers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch customers");
        }
    }
}

#[post("/find_customer")]
pub async fn find_customer(pool: web::Data<Pool>, form: web::Json<EmailForm>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    let email = form.email.clone();
    match Customer::find_by_email(&**client, &email).await {
        Ok(customer) => HttpResponse::Ok().json(customer),
        Err(_) => HttpResponse::InternalServerError().json("customer not found"),
    }
}
