use crate::{cert_manager::CertManager, error::Result};
use rustls::crypto::ring::sign::any_supported_type;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

pub struct DynamicCertResolver {
    cert_manager: Arc<CertManager>,
    /// Host from the CONNECT line, used when the client sends no DNS SNI (raw IP).
    fallback_name: String,
}

impl std::fmt::Debug for DynamicCertResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicCertResolver").finish()
    }
}

impl DynamicCertResolver {
    pub fn new(cert_manager: Arc<CertManager>) -> Self {
        Self::with_fallback(cert_manager, String::new())
    }

    pub fn with_fallback(cert_manager: Arc<CertManager>, fallback_name: String) -> Self {
        Self {
            cert_manager,
            fallback_name,
        }
    }
}

impl ResolvesServerCert for DynamicCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let server_name = client_hello
            .server_name()
            .map(|name| name.to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.fallback_name.clone());
        if server_name.is_empty() {
            return None;
        }
        let pair = self.cert_manager.generate_cert(&server_name).ok()?;
        let signing_key = any_supported_type(&pair.1).ok()?;
        let certified = CertifiedKey::new(vec![pair.0.clone()], signing_key);
        Some(Arc::new(certified))
    }
}

pub struct TlsInterceptor {
    pub cert_manager: Arc<CertManager>,
    pub acceptor: TlsAcceptor,
}

impl TlsInterceptor {
    pub fn new(cert_manager: Arc<CertManager>) -> Result<Self> {
        crate::connection_pool::install_crypto_provider();
        let acceptor = Self::build_acceptor(cert_manager.clone(), String::new());
        Ok(Self {
            cert_manager,
            acceptor,
        })
    }

    /// Acceptor that can mint a certificate for `fallback_host` when the
    /// client does not send a DNS server name.
    pub fn acceptor_for(&self, fallback_host: &str) -> TlsAcceptor {
        Self::build_acceptor(self.cert_manager.clone(), fallback_host.to_string())
    }

    fn build_acceptor(cert_manager: Arc<CertManager>, fallback_host: String) -> TlsAcceptor {
        let resolver = Arc::new(DynamicCertResolver::with_fallback(
            cert_manager,
            fallback_host,
        ));
        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        // Prefer HTTP/2, fall back to HTTP/1.1. WebSocket upgrades need HTTP/1.1.
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        TlsAcceptor::from(Arc::new(config))
    }

    /// Get the negotiated ALPN protocol from the TLS connection
    /// Returns Some("h2") for HTTP/2 or Some("http/1.1") for HTTP/1.1
    pub fn get_alpn_protocol(
        stream: &tokio_rustls::server::TlsStream<impl tokio::io::AsyncRead + tokio::io::AsyncWrite>,
    ) -> Option<String> {
        stream
            .get_ref()
            .1
            .alpn_protocol()
            .and_then(|proto| String::from_utf8(proto.to_vec()).ok())
    }
}
