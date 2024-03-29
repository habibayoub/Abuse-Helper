use actix_web::{get, web, HttpResponse};
use whois_rust::{WhoIs, WhoIsLookupOptions};

#[get("/whois/{address}")]
pub async fn whois(path: web::Path<String>) -> HttpResponse {
    let whois = match WhoIs::from_path_async("./data/whois_servers.json").await {
        Ok(whois) => whois,
        Err(err) => {
            log::debug!("unable to load whois servers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to load whois servers");
        }
    };

    let address = path.into_inner();
    let lookup = match WhoIsLookupOptions::from_string(&address) {
        Ok(lookup) => lookup,
        Err(err) => {
            log::debug!("unable to parse domain for whois lookup: {:?}", err);
            return HttpResponse::InternalServerError()
                .json("unable to parse domain for whois lookup");
        }
    };

    match whois.lookup_async(lookup).await {
        Ok(record) => HttpResponse::Ok().json(record),
        Err(err) => {
            log::debug!("unable to fetch whois: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(format!("unable to fetch whois record for {}", &address));
        }
    }
}
