use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use deadpool_postgres::Pool;

mod customer;
mod postgres;


#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String
}

#[post("/authenticate")]
async fn authenticate(
    pool: web::Data<Pool>,
    form: web::Json<LoginForm>,
) -> HttpResponse {
    let credentials = form.credentials.clone();

    // test credentials against AD and return 'status' key with values true/false
    
}

#[get("/customers")]
async fn list_customers(pool: web::Data<Pool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match customer::Customer::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch customers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch customers");
        }
    }
}

#[post("/find_customer")]
async fn find_customer(
    pool: web::Data<Pool>,
    form: web::Json<customer::EmailForm>,
) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    let email = form.email.clone();
    match customer::Customer::find_by_email(&**client, &email).await {
        Ok(customer) => HttpResponse::Ok().json(customer),
        Err(_) => HttpResponse::InternalServerError().json("customer not found"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            .service(list_customers)
            .service(find_customer)
    })
    .bind(&address)?
    .run()
    .await
}
