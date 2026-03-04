use std::{env, path::Path};

use sqlx::{PgPool, postgres::PgPoolOptions};
use thiserror::Error;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn from_env() -> Result<Self, AppStateError> {
        let database_url = env::var("DATABASE_URL")?;
        Self::from_database_url(&database_url).await
    }

    pub async fn from_database_url(database_url: &str) -> Result<Self, AppStateError> {
        let db = PgPoolOptions::new().connect(database_url).await?;
        run_migrations(&db).await?;
        Ok(Self { db })
    }
}

pub async fn run_migrations(db: &PgPool) -> Result<(), AppStateError> {
    let migrator = sqlx::migrate::Migrator::new(Path::new("./migrations")).await?;
    migrator.run(db).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("DATABASE_URL is not configured")]
    Env(#[from] env::VarError),
    #[error("database connection failed")]
    Database(#[from] sqlx::Error),
    #[error("database migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),
}
