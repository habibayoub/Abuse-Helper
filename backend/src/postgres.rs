use deadpool_postgres::{Config, Pool};
use tokio_postgres::Error;
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

pub async fn run_migrations(pool: &Pool) -> Result<(), Error> {
    let client = pool.get().await.expect("couldn't get postgres client");

    // Create migrations table if it doesn't exist
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )",
            &[],
        )
        .await?;

    for (name, sql) in SCRIPTS_UP.iter() {
        // Check if migration has been applied
        let row = client
            .query_one("SELECT COUNT(*) FROM migrations WHERE name = $1", &[name])
            .await?;
        let count: i64 = row.get(0);

        if count == 0 {
            // Run the migration
            client.batch_execute(sql).await?;

            // Mark migration as applied
            client
                .execute("INSERT INTO migrations (name) VALUES ($1)", &[name])
                .await?;

            println!("Applied migration: {}", name);
        } else {
            println!("Skipping migration: {} (already applied)", name);
        }
    }

    Ok(())
}
