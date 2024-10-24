use deadpool_postgres::{Config, Pool};
use once_cell::sync::OnceCell;
use std::env;
use tokio::runtime::Runtime;
use tokio_postgres::NoTls;

static DB_POOL: OnceCell<Pool> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn get_db_pool() -> &'static Pool {
    DB_POOL.get().expect("Database pool not initialized")
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().expect("Runtime not initialized")
}

pub fn initialize_tests() {
    let runtime = Runtime::new().expect("Failed to create runtime");
    let _ = RUNTIME.set(runtime);

    get_runtime().block_on(async {
        let pool = setup_test_db().await;
        clear_database(&pool)
            .await
            .expect("Failed to clear database");
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");
        let _ = DB_POOL.set(pool);
    });

    // Set JWT_SECRET for tests
    env::set_var("JWT_SECRET", "dw3QKXwLxzufwTHymvWjfdMfMcDDlckc");
}

pub async fn setup_test_db() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("test_db".to_string());
    cfg.user = Some("test_user".to_string());
    cfg.password = Some("test_password".to_string());

    cfg.create_pool(NoTls).expect("Failed to create pool")
}

pub async fn clear_database(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Drop all tables and sequences
    client
        .batch_execute(
            "
            DO $$ 
            DECLARE 
                r RECORD;
            BEGIN
                FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = current_schema()) LOOP
                    EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
                END LOOP;
                FOR r IN (SELECT sequencename FROM pg_sequences WHERE schemaname = current_schema()) LOOP
                    EXECUTE 'DROP SEQUENCE IF EXISTS ' || quote_ident(r.sequencename) || ' CASCADE';
                END LOOP;
            END $$;
            "
        )
        .await?;

    Ok(())
}

pub async fn run_migrations(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Drop existing sequences
    client
        .batch_execute(
            "
            DO $$ 
            DECLARE 
                r RECORD;
            BEGIN
                FOR r IN (SELECT sequencename FROM pg_sequences WHERE schemaname = current_schema()) LOOP
                    EXECUTE 'DROP SEQUENCE IF EXISTS ' || quote_ident(r.sequencename) || ' CASCADE';
                END LOOP;
            END $$;
            "
        )
        .await?;

    // Create migrations table if it doesn't exist
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;

    let migration_files = [
        (
            "0000_create_migrations_table.sql",
            include_str!("../../migrations/0000_create_migrations_table.sql"),
        ),
        (
            "0001_create-customers.sql",
            include_str!("../../migrations/0001_create-customers.sql"),
        ),
        (
            "0002_create-nctns.sql",
            include_str!("../../migrations/0002_create-nctns.sql"),
        ),
        (
            "0003_create_users.sql",
            include_str!("../../migrations/0003_create_users.sql"),
        ),
        (
            "0004_create_blacklisted_tokens.sql",
            include_str!("../../migrations/0004_create_blacklisted_tokens.sql"),
        ),
        (
            "0005_create_user_logs.sql",
            include_str!("../../migrations/0005_create_user_logs.sql"),
        ),
    ];

    for (migration_name, migration_sql) in migration_files.iter() {
        // Check if migration has already been applied
        let already_applied = client
            .query_one(
                "SELECT COUNT(*) FROM migrations WHERE name = $1",
                &[migration_name],
            )
            .await?
            .get::<_, i64>(0)
            > 0;

        if !already_applied {
            // Run the migration
            client.batch_execute(migration_sql).await?;

            // Record that the migration has been applied
            client
                .execute(
                    "INSERT INTO migrations (name) VALUES ($1)",
                    &[migration_name],
                )
                .await?;

            println!("Applied migration: {}", migration_name);
        } else {
            println!("Skipping already applied migration: {}", migration_name);
        }
    }

    Ok(())
}
