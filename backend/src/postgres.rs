use deadpool_postgres::{Config, Pool};
use tokio_postgres::Error;
use tokio_postgres::NoTls;

/// Scripts to run for the up migration
///
/// # Migration Order
/// 1. Create customers table
/// 2. Create NCTNS (Network and Cyber Threat Notification System) table
/// 3. Create users table
/// 4. Create blacklisted tokens table
/// 5. Create user logs table
/// 6. Create emails table
/// 7. Add is_sent column to emails
/// 8. Create tickets table
/// 9. Create email_tickets junction table
/// 10. Remove email_id from tickets
/// 11. Change email_id to UUID type
///
/// # Migration Safety
/// - Migrations are executed in order
/// - Each migration is tracked in the migrations table
/// - Duplicate migrations are skipped
const SCRIPTS_UP: [(&str, &str); 11] = [
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
    (
        "0006_create_emails_table",
        include_str!("../migrations/0006_create_emails_table.sql"),
    ),
    (
        "0007_add_is_sent_to_emails",
        include_str!("../migrations/0007_add_is_sent_to_emails.sql"),
    ),
    (
        "0008_create_tickets_table",
        include_str!("../migrations/0008_create_tickets_table.sql"),
    ),
    (
        "0009_create_email_tickets_table",
        include_str!("../migrations/0009_create_email_tickets_table.sql"),
    ),
    (
        "0010_remove_email_id_from_tickets",
        include_str!("../migrations/0010_remove_email_id_from_tickets.sql"),
    ),
    (
        "0011_change_email_id_to_uuid",
        include_str!("../migrations/0011_change_email_id_to_uuid.sql"),
    ),
];

/// Create a new configuration from environment variables
///
/// # Environment Variables
/// - `PG_HOST`: Database host address
/// - `PG_DBNAME`: Database name
/// - `PG_USER`: Database user
/// - `PG_PASSWORD`: Database password
///
/// # Returns
/// Configuration object for database pool creation
fn create_config() -> Config {
    // Create a new configuration
    let mut cfg = Config::new();
    // Set the host if it's available
    if let Ok(host) = std::env::var("PG_HOST") {
        cfg.host = Some(host);
    }
    // Set the database name if it's available
    if let Ok(dbname) = std::env::var("PG_DBNAME") {
        cfg.dbname = Some(dbname);
    }
    // Set the user if it's available
    if let Ok(user) = std::env::var("PG_USER") {
        cfg.user = Some(user);
    }
    // Set the password if it's available
    if let Ok(password) = std::env::var("PG_PASSWORD") {
        cfg.password = Some(password);
    }
    // Return the configuration
    cfg
}

/// Create a new database connection pool
///
/// # Returns
/// Configured connection pool ready for use
///
/// # Panics
/// If pool creation fails due to invalid configuration
///
/// # Configuration
/// Uses environment variables through create_config()
pub fn create_pool() -> Pool {
    create_config()
        .create_pool(NoTls)
        .expect("couldn't create postgres pool")
}

/// Executes database migrations in order
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<(), Error>` - Success or database error
///
/// # Migration Process
/// 1. Creates migrations table if not exists
/// 2. Checks each migration's status
/// 3. Executes pending migrations
/// 4. Records successful migrations
pub async fn run_migrations(pool: &Pool) -> Result<(), Error> {
    // Get a client from the pool
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

    // Iterate over the migration scripts
    for (name, sql) in SCRIPTS_UP.iter() {
        // Check if migration has been applied
        let row = client
            .query_one("SELECT COUNT(*) FROM migrations WHERE name = $1", &[name])
            .await?;

        // Get the count of applied migrations
        let count: i64 = row.get(0);

        // If the migration hasn't been applied, run it
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
