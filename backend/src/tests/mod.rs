mod auth_tests;
mod common;
mod customer_tests;
mod nctns_tests;
mod whois_tests;

#[cfg(test)]
#[actix_rt::test]
async fn init() {
    common::initialize_tests().await;
}
