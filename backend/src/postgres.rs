use deadpool_postgres::{Config, Pool};
use tokio_postgres::NoTls;

/// Scripts to run for the up migration
const SCRIPTS_UP: [(&str, &str); 5] = [
    (
        "0001_create-customers",
        include_str!("../migrations/0001_create-customers.sql"),
    ),
    (
        "0002_create-nctns",
        include_str!("../migrations/0002_create-nctns.sql"),
    ),
    (
        "0003_create_users",
        include_str!("../migrations/0003_create_users.sql"),
    ),
    (
        "0004_create_blacklisted_tokens",
        include_str!("../migrations/0004_create_blacklisted_tokens.sql"),
    ),
    (
        "0005_create_user_logs",
        include_str!("../migrations/0005_create_user_logs.sql"),
    ),
];

/// Create a new configuration from environment variables
fn create_config() -> Config {
    let mut cfg = Config::new();
    if let Ok(host) = std::env::var("PG_HOST") {
        cfg.host = Some(host);
    }
    if let Ok(dbname) = std::env::var("PG_DBNAME") {
        cfg.dbname = Some(dbname);
    }
    if let Ok(user) = std::env::var("PG_USER") {
        cfg.user = Some(user);
    }
    if let Ok(password) = std::env::var("PG_PASSWORD") {
        cfg.password = Some(password);
    }
    cfg
}

/// Create a new database pool
pub fn create_pool() -> Pool {
    create_config()
        .create_pool(NoTls)
        .expect("couldn't create postgres pool")
}

/// Run the up migrations
pub async fn migrate_up(pool: &Pool) {
    let client = pool.get().await.expect("couldn't get postgres client");
    for (name, script) in SCRIPTS_UP.iter() {
        client
            .batch_execute(script)
            .await
            .unwrap_or_else(|e| panic!("Failed to execute migration {}: {:?}", name, e));
        println!("Executed migration: {}", name);
    }
}
