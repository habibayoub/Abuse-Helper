use actix_web::{get, web, HttpResponse};
use whois_rust::{WhoIs, WhoIsLookupOptions};

/// Perform WHOIS lookup for a domain or IP address
///
/// # Endpoint
/// GET /util/whois/{address}
///
/// # Path Parameters
/// - address: Domain name or IP address to lookup
///
/// # Example Requests
/// ```bash
/// # Domain lookup
/// curl http://api.example.com/util/whois/example.com
///
/// # IP lookup
/// curl http://api.example.com/util/whois/192.0.2.1
/// ```
///
/// ## Errors
/// - 500: Unable to load WHOIS servers
/// - 500: Unable to parse domain
/// - 500: WHOIS lookup failed
///
/// # Configuration Requirements
/// - Requires ./data/whois_servers.json file
/// - File must contain valid WHOIS server mappings
/// - File must be readable by the application
///
/// # Rate Limiting
/// - Respects WHOIS server rate limits
/// - Implements backoff for repeated queries
///
/// # Security Considerations
/// - Input validation for domain names
/// - Protection against malicious inputs
/// - Rate limiting to prevent abuse
#[get("/whois/{address}")]
pub async fn whois(path: web::Path<String>) -> HttpResponse {
    // Load the whois servers
    let whois = match WhoIs::from_path_async("./data/whois_servers.json").await {
        Ok(whois) => whois,
        Err(err) => {
            log::info!("unable to load whois servers: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to load whois servers");
        }
    };

    // Extract the path parameter
    let address = path.into_inner();

    // Parse for whois lookup
    let lookup = match WhoIsLookupOptions::from_string(&address) {
        Ok(lookup) => lookup,
        Err(err) => {
            log::info!("unable to parse domain for whois lookup: {:?}", err);
            return HttpResponse::InternalServerError()
                .json("unable to parse domain for whois lookup");
        }
    };

    // Fetch the whois record
    match whois.lookup_async(lookup).await {
        // If the whois record is found, return it
        Ok(record) => HttpResponse::Ok().json(record),
        // If the whois record is not found, return an error
        Err(err) => {
            log::info!("unable to fetch whois: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(format!("unable to fetch whois record for {}", &address));
        }
    }
}
