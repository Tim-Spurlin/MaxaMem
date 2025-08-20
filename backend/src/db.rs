use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(_pool: &PgPool) -> Result<()> {
    // TODO: Run migrations when database is available
    tracing::info!("Database migrations skipped - run manually");
    Ok(())
}