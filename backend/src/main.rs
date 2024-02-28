mod models;
mod postgres;
mod routes;

use actix_web::{web, App, HttpServer};
use routes::customer::{find_customer, list_customers};

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

// #[post("/authenticate")]
// async fn authenticate(pool: web::Data<Pool>, form: web::Json<LoginForm>) -> HttpResponse {
//     let username = form.username.clone();
//     let password = form.password.clone();

//     // test credentials against AD and return 'status' key with values true/false
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            // .service(authenticate)
            .service(list_customers)
            .service(find_customer)
    })
    .bind(&address)?
    .run()
    .await
}
