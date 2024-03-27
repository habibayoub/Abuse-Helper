use actix_web::{get, post, web, HttpResponse};
use deadpool_postgres::Pool;

use crate::models::customer::{Customer, LookUpForm};

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
pub async fn find_customer(pool: web::Data<Pool>, form: web::Json<LookUpForm>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };

    match (form.email.clone(), form.ip.clone(), form.id.clone()) {
    (Some(email), _, _) => {
            match Customer::find_by_email(&**client, &email).await {
                Ok(customer) => return HttpResponse::Ok().json(customer),
                Err(_) => return HttpResponse::InternalServerError().json("customer not found"),
            };
    },
    (_, Some(ip), _) => {
            match Customer::find_by_ip(&**client, &ip).await {
                Ok(customer) => return HttpResponse::Ok().json(customer),
                Err(_) => return HttpResponse::InternalServerError().json("customer not found"),
            }
    },
    (_, _, Some(id)) => {
            match Customer::find_by_id(&**client, id).await {
                Ok(customer) => HttpResponse::Ok().json(customer),
                Err(_) => HttpResponse::InternalServerError().json("customer not found"),
            }
    },
    _ => HttpResponse::InternalServerError().json("please enter a valid lookup value [email | ip]")
    }


}
