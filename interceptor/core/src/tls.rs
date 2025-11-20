use crate::{cert_manager::CertManager, error::Result};
use rustls::crypto::ring::sign::any_supported_type;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

pub struct DynamicCertResolver {
    cert_manager: Arc<CertManager>,
}

impl std::fmt::Debug for DynamicCertResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicCertResolver").finish()
    }
}

impl DynamicCertResolver {
    pub fn new(cert_manager: Arc<CertManager>) -> Self {
        Self { cert_manager }
    }
}

impl ResolvesServerCert for DynamicCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let server_name = client_hello.server_name()?.to_string();
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
        let resolver = Arc::new(DynamicCertResolver::new(cert_manager.clone()));
        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);
        config.alpn_protocols = vec![b"http/1.1".to_vec(), b"h2".to_vec()];
        let acceptor = TlsAcceptor::from(Arc::new(config));
        Ok(Self {
            cert_manager,
            acceptor,
        })
    }
}
