use http_body_util::Full;
use hyper::body::Bytes;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::TokioExecutor;
use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;
use std::sync::Arc;

pub type ProxyBody = Full<Bytes>;
pub type HttpClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, ProxyBody>;

/// Install a process-wide rustls CryptoProvider when none is selected yet.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn build_client(http2: bool, extra_roots: Option<&[CertificateDer<'static>]>) -> HttpClient {
    install_crypto_provider();
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);

    let builder = if let Some(certs) = extra_roots {
        let mut store = RootCertStore::empty();
        for cert in certs {
            store
                .add(cert.clone())
                .expect("extra root certificate is valid");
        }
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(store)
            .with_no_client_auth();
        HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_or_http()
            .enable_http1()
    } else {
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("load native roots")
            .https_or_http()
            .enable_http1()
    };
    let https = if http2 {
        builder.enable_http2().wrap_connector(connector)
    } else {
        builder.wrap_connector(connector)
    };
    Client::builder(TokioExecutor::new()).build(https)
}

#[derive(Clone)]
pub struct ConnectionPool {
    client: Arc<HttpClient>,
    /// HTTP/1.1 only. WebSocket upgrades are not carried on HTTP/2.
    http1: Arc<HttpClient>,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            client: Arc::new(build_client(true, None)),
            http1: Arc::new(build_client(false, None)),
        }
    }

    /// Upstream client that trusts only `roots` (a test origin CA). Production
    /// traffic keeps [`ConnectionPool::new`] and the platform trust store.
    pub fn trusting_extra(roots: Vec<CertificateDer<'static>>) -> Self {
        Self {
            client: Arc::new(build_client(true, Some(&roots))),
            http1: Arc::new(build_client(false, Some(&roots))),
        }
    }

    pub fn client(&self) -> Arc<HttpClient> {
        self.client.clone()
    }

    /// Client that speaks only HTTP/1.1, so a `101 Switching Protocols`
    /// WebSocket upgrade is not negotiated away as HTTP/2.
    pub fn http1_client(&self) -> Arc<HttpClient> {
        self.http1.clone()
    }
}
