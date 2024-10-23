mod auth_tests;
mod common;
mod customer_tests;
mod nctns_tests;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    common::initialize_tests();
}
