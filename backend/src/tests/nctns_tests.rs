use super::common;
use crate as abuse_helper;
use abuse_helper::models::nctns::NCTNS;
use abuse_helper::routes::nctns;
use actix_web::{test, web, App};
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;

#[actix_rt::test]
async fn test_list_nctns() {
    let pool = common::get_db_pool().clone();

    // Clear the nctns table before the test
    common::clear_table(&pool, "nctns")
        .await
        .expect("Failed to clear nctns table");

    // Insert test data
    let client = pool.get().await.expect("Failed to get client");
    let uuid = Uuid::new_v4();
    let now: DateTime<Utc> = Utc::now();

    client
        .execute(
            "INSERT INTO nctns (uuid, source_time, time, ip, reverse_dns, domain_name, asn, as_name, category, type, malware_family, vulnerability, tag, source_name, comment, description, description_url, destination_ip, destination_port, port, protocol, transport_protocol, http_request, user_agent, username, url, destination_domain_name, status, observation_time, source_feed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30)",
            &[
                &uuid,
                &"source1",
                &now,
                &"192.168.1.1",
                &"example.com",
                &"example.com",
                &"AS12345",
                &"Example AS",
                &"Category1",
                &"Type1",
                &"MalwareFamily1",
                &"Vulnerability1",
                &"Tag1",
                &"SourceName1",
                &"Comment1",
                &"Description1",
                &"http://example.com/description1",
                &"192.168.2.1",
                &80i32,
                &8080i32,
                &"TCP",
                &"HTTP",
                &"GET /page1",
                &"Mozilla/5.0",
                &"user1",
                &"http://example.com/page1",
                &"example.com",
                &"Success",
                &now,
                &"feed1",
            ]
        )
        .await
        .expect("Failed to insert test data");

    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/nctns").service(nctns::list)),
    )
    .await;

    let req = test::TestRequest::get().uri("/nctns").to_request();
    let resp = test::call_service(&mut app, req).await;

    println!("Response status: {:?}", resp.status());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert body to string");
    println!("Response body: {}", body_str);

    let parsed_resp: Result<Vec<NCTNS>, serde_json::Error> = serde_json::from_str(&body_str);

    match parsed_resp {
        Ok(nctns_list) => {
            assert!(!nctns_list.is_empty());
            assert_eq!(nctns_list[0].uuid, uuid);
            assert_eq!(nctns_list[0].source_name, "SourceName1");
            assert_eq!(nctns_list[0].ip, "192.168.1.1");
            assert_eq!(nctns_list[0].domain_name, "example.com");
        }
        Err(e) => {
            panic!(
                "Failed to parse response: {:?}\nResponse body: {}",
                e, body_str
            );
        }
    }
}
