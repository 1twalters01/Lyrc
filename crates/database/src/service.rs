use configuration::config::Config;
use sqlx::{
    Pool,
    sqlite::{Sqlite, SqlitePoolOptions},
};

pub struct DatabaseService {
    pool: Pool<Sqlite>,
}

impl DatabaseService {
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn from_url(database_url: &str, max_connections: u32) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn new(config: &Config) -> Result<Self, sqlx::Error> {
        Self::from_url(&config.database_url, config.max_connections).await
    }

    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }
}
