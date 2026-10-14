//! The durable audit adapter. Payloads end here; the journal never forwards them to another RPC.

use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<StoreError> for tonic::Status {
    fn from(error: StoreError) -> Self {
        tracing::error!(%error, "journal write failed");
        Self::unavailable("the audit store is unavailable")
    }
}

#[derive(Clone)]
pub struct Journal {
    pool: PgPool,
}

impl Journal {
    pub async fn connect(url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new().max_connections(16).connect(url).await?;
        Ok(Self { pool })
    }

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), StoreError> {
        sqlx::raw_sql(include_str!("../migrations/0001_migration.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn append(&self, domain: &str, method: &str, tenant: &str, payload: Vec<u8>) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO acme_audit (domain, method, tenant_id, payload) VALUES ($1, $2, $3, $4)")
            .bind(domain)
            .bind(method)
            .bind(tenant)
            .bind(payload)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
