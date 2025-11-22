use http_body_util::Full;
use hyper::body::Bytes;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::TokioExecutor;
use std::sync::Arc;

pub type ProxyBody = Full<Bytes>;
pub type HttpClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, ProxyBody>;

#[derive(Clone)]
pub struct ConnectionPool {
    client: Arc<HttpClient>,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPool {
    pub fn new() -> Self {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("load native roots")
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .wrap_connector(connector);
        let client = Client::builder(TokioExecutor::new()).build(https);
        Self {
            client: Arc::new(client),
        }
    }

    pub fn client(&self) -> Arc<HttpClient> {
        self.client.clone()
    }
}
