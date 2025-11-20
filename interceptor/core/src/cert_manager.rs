use crate::error::Result;
use dashmap::DashMap;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, IsCa, KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

const CERT_FILE: &str = "ca_cert.pem";
const KEY_FILE: &str = "ca_key.pem";

pub struct CertManager {
    ca_cert: Certificate,
    cert_cache: DashMap<String, Arc<(CertificateDer<'static>, PrivateKeyDer<'static>)>>,
    ca_dir: PathBuf,
}

impl CertManager {
    pub fn new() -> Result<Self> {
        let ca_dir = Self::default_ca_dir();
        std::fs::create_dir_all(&ca_dir)?;
        let ca_pem = ca_dir.join(CERT_FILE);
        let ca_key = ca_dir.join(KEY_FILE);

        let ca_cert = if ca_pem.exists() && ca_key.exists() {
            Self::load_existing(&ca_pem, &ca_key)?
        } else {
            let cert = Self::create_ca()?;
            std::fs::write(&ca_pem, cert.serialize_pem()?)?;
            std::fs::write(&ca_key, cert.serialize_private_key_pem())?;
            cert
        };

        Ok(Self {
            ca_cert,
            cert_cache: DashMap::new(),
            ca_dir,
        })
    }

    fn default_ca_dir() -> PathBuf {
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        base.join(".interceptor/ca")
    }

    fn create_ca() -> Result<Certificate> {
        let mut params = CertificateParams::new(Vec::new());
        params
            .distinguished_name
            .push(DnType::CommonName, "Interceptor Proxy CA");
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Interceptor");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        params.not_before = OffsetDateTime::now_utc() - Duration::days(1);
        params.not_after = OffsetDateTime::now_utc() + Duration::days(3650);
        Certificate::from_params(params).map_err(Into::into)
    }

    fn load_existing(cert_path: &Path, key_path: &Path) -> Result<Certificate> {
        let cert = std::fs::read_to_string(cert_path)?;
        let key = std::fs::read_to_string(key_path)?;
        let key_pair = KeyPair::from_pem(&key)?;
        let mut params = CertificateParams::from_ca_cert_pem(&cert, key_pair)?;
        params
            .distinguished_name
            .push(DnType::CommonName, "Interceptor Proxy CA");
        Certificate::from_params(params).map_err(Into::into)
    }

    pub fn generate_cert(
        &self,
        domain: &str,
    ) -> Result<Arc<(CertificateDer<'static>, PrivateKeyDer<'static>)>> {
        if let Some(entry) = self.cert_cache.get(domain) {
            return Ok(entry.value().clone());
        }

        let mut params = CertificateParams::new(vec![domain.to_string()]);
        params.distinguished_name.push(DnType::CommonName, domain);
        let cert = Certificate::from_params(params)?;
        let key = cert.serialize_private_key_der();
        let der = cert.serialize_der_with_signer(&self.ca_cert)?;
        let key = PrivatePkcs8KeyDer::from(key);
        let pair = Arc::new((CertificateDer::from(der), PrivateKeyDer::from(key)));
        self.cert_cache.insert(domain.to_string(), pair.clone());
        Ok(pair)
    }

    pub fn export_ca_cert(&self, output: &Path) -> Result<()> {
        let pem = self.ca_cert.serialize_pem()?;
        std::fs::write(output, pem)?;
        Ok(())
    }

    pub fn ca_pem(&self) -> Result<String> {
        Ok(self.ca_cert.serialize_pem()?)
    }

    pub fn ca_directory(&self) -> &Path {
        &self.ca_dir
    }
}
