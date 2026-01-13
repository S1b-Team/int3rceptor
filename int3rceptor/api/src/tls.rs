use anyhow::{Context, Result};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tracing::{info, warn};

/// TLS configuration for the API server
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the certificate file (PEM format)
    pub cert_path: PathBuf,
    /// Path to the private key file (PEM format)
    pub key_path: PathBuf,
    /// Enable HTTP/2
    pub http2: bool,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_path: impl AsRef<Path>, key_path: impl AsRef<Path>) -> Self {
        Self {
            cert_path: cert_path.as_ref().to_path_buf(),
            key_path: key_path.as_ref().to_path_buf(),
            http2: true,
        }
    }

    /// Load TLS configuration from environment variables
    pub fn from_env() -> Option<Self> {
        let cert_path = std::env::var("TLS_CERT_PATH").ok()?;
        let key_path = std::env::var("TLS_KEY_PATH").ok()?;

        Some(Self {
            cert_path: PathBuf::from(cert_path),
            key_path: PathBuf::from(key_path),
            http2: std::env::var("TLS_HTTP2")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true),
        })
    }

    /// Validate that certificate and key files exist
    pub fn validate(&self) -> Result<()> {
        if !self.cert_path.exists() {
            anyhow::bail!("TLS certificate file not found: {:?}", self.cert_path);
        }

        if !self.key_path.exists() {
            anyhow::bail!("TLS private key file not found: {:?}", self.key_path);
        }

        Ok(())
    }

    /// Build rustls configuration
    pub async fn build_rustls_config(&self) -> Result<RustlsConfig> {
        self.validate()?;

        RustlsConfig::from_pem_file(&self.cert_path, &self.key_path)
            .await
            .context("Failed to load TLS certificate and key")
    }
}

/// Serve the application with TLS support
pub async fn serve_with_tls(app: Router, addr: SocketAddr, tls_config: TlsConfig) -> Result<()> {
    let rustls_config = tls_config.build_rustls_config().await?;

    info!("Starting HTTPS server on https://{} with TLS enabled", addr);
    info!("Certificate: {:?}", tls_config.cert_path);
    info!("Private Key: {:?}", tls_config.key_path);
    info!("HTTP/2: {}", tls_config.http2);

    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("TLS server failed")
}

/// Serve the application without TLS (HTTP only)
pub async fn serve_without_tls(app: Router, addr: SocketAddr) -> Result<()> {
    warn!(
        "Starting HTTP server on http://{} WITHOUT TLS encryption",
        addr
    );
    warn!("⚠️  TLS is disabled. This is insecure for production use!");

    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("HTTP server failed")
}

/// Auto-detect TLS configuration and serve appropriately
pub async fn serve_auto(app: Router, addr: SocketAddr) -> Result<()> {
    match TlsConfig::from_env() {
        Some(tls_config) => {
            info!("TLS configuration detected from environment");
            serve_with_tls(app, addr, tls_config).await
        }
        None => {
            info!("No TLS configuration found, starting in HTTP mode");
            serve_without_tls(app, addr).await
        }
    }
}

/// Generate self-signed certificate for development (requires rcgen)
#[cfg(feature = "dev-certs")]
pub mod dev {
    use super::*;
    use rcgen::{Certificate, CertificateParams, DistinguishedName};
    use std::fs;

    pub fn generate_self_signed_cert(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> Result<()> {
        let mut params = CertificateParams::default();
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "interceptor-api.local");
        params.subject_alt_names = vec![
            rcgen::SanType::DnsName("localhost".to_string()),
            rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        ];

        let cert = Certificate::from_params(params)?;

        fs::write(cert_path.as_ref(), cert.serialize_pem()?)?;
        fs::write(key_path.as_ref(), cert.serialize_private_key_pem())?;

        info!("Generated self-signed certificate");
        info!("  Certificate: {:?}", cert_path.as_ref());
        info!("  Private Key: {:?}", key_path.as_ref());
        warn!("⚠️  Self-signed certificates should only be used for development!");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_creation() {
        let config = TlsConfig::new("cert.pem", "key.pem");
        assert_eq!(config.cert_path, PathBuf::from("cert.pem"));
        assert_eq!(config.key_path, PathBuf::from("key.pem"));
        assert!(config.http2);
    }

    #[test]
    fn test_tls_config_validation_fails_missing_files() {
        let config = TlsConfig::new("nonexistent_cert.pem", "nonexistent_key.pem");
        assert!(config.validate().is_err());
    }
}
