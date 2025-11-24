use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProxyError>;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::http::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] hyper::http::uri::InvalidUri),
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
    #[error("Certificate error: {0}")]
    Certificate(#[from] rcgen::Error),
    #[error("HTTP client error: {0}")]
    Client(#[from] hyper_util::client::legacy::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Other: {0}")]
    Other(String),
}

impl From<anyhow::Error> for ProxyError {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value.to_string())
    }
}
