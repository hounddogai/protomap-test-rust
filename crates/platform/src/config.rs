//! Environment configuration for servers and scheduled jobs.

use std::{env, net::SocketAddr, time::Duration};
use tonic::transport::Channel;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("ACME_DATABASE_URL must be set")]
    Database,
    #[error("ACME_LISTEN is invalid: {0}")]
    Listen(#[from] std::net::AddrParseError),
    #[error("the mesh endpoint is invalid: {0}")]
    Mesh(#[from] http::uri::InvalidUri),
}

#[derive(Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub database_url: String,
    pub mesh: String,
}

impl Config {
    pub fn load(port: u16) -> Result<Self, ConfigError> {
        Ok(Self {
            listen: env::var("ACME_LISTEN")
                .unwrap_or_else(|_| format!("127.0.0.1:{port}"))
                .parse()?,
            database_url: env::var("ACME_DATABASE_URL").map_err(|_| ConfigError::Database)?,
            mesh: env::var("ACME_MESH").unwrap_or_else(|_| "http://127.0.0.1:15001".to_owned()),
        })
    }

    /// The sidecar routes service paths to their owning domain. Channels are cheap to clone.
    pub fn channel(&self) -> Result<Channel, ConfigError> {
        Ok(Channel::from_shared(self.mesh.clone())?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(15))
            .connect_lazy())
    }
}

pub fn logging() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
