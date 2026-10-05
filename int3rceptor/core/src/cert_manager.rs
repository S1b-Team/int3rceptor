use crate::error::Result;
use dashmap::DashMap;
use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

const CERT_FILE: &str = "ca_cert.pem";
const KEY_FILE: &str = "ca_key.pem";

pub struct CertManager {
    /// PEM of the on-disk CA. Reused across restarts; never replaced when the files exist.
    ca_pem: String,
    ca_key_pair: KeyPair,
    cert_cache: DashMap<String, Arc<(CertificateDer<'static>, PrivateKeyDer<'static>)>>,
    ca_dir: PathBuf,
}

impl CertManager {
    pub fn new() -> Result<Self> {
        let ca_dir = Self::default_ca_dir();
        std::fs::create_dir_all(&ca_dir)?;
        let ca_pem_path = ca_dir.join(CERT_FILE);
        let ca_key_path = ca_dir.join(KEY_FILE);

        let (ca_pem, ca_key_pair) = if ca_pem_path.exists() && ca_key_path.exists() {
            Self::load_existing(&ca_pem_path, &ca_key_path)?
        } else {
            let (_params, cert, key_pair) = Self::create_ca()?;
            let pem = cert.pem();
            std::fs::write(&ca_pem_path, &pem)?;
            std::fs::write(&ca_key_path, key_pair.serialize_pem())?;
            (pem, key_pair)
        };

        Ok(Self {
            ca_pem,
            ca_key_pair,
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

    fn create_ca() -> Result<(CertificateParams, rcgen::Certificate, KeyPair)> {
        let mut params = CertificateParams::new(vec!["Interceptor Proxy CA".to_string()])?;
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
        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;
        Ok((params, cert, key_pair))
    }

    fn load_existing(cert_path: &Path, key_path: &Path) -> Result<(String, KeyPair)> {
        let pem = std::fs::read_to_string(cert_path)?;
        let key_pem = std::fs::read_to_string(key_path)?;
        let key_pair = KeyPair::from_pem(&key_pem)?;
        // Fail closed if the saved CA cannot be used to sign leaf certificates.
        let _ = Issuer::from_ca_cert_pem(&pem, &key_pair)?;
        Ok((pem, key_pair))
    }

    pub fn generate_cert(
        &self,
        domain: &str,
    ) -> Result<Arc<(CertificateDer<'static>, PrivateKeyDer<'static>)>> {
        if let Some(entry) = self.cert_cache.get(domain) {
            return Ok(entry.value().clone());
        }

        let mut params = CertificateParams::new(vec![domain.to_string()])?;
        params.distinguished_name.push(DnType::CommonName, domain);
        let key_pair = KeyPair::generate()?;

        let issuer = Issuer::from_ca_cert_pem(&self.ca_pem, &self.ca_key_pair)?;
        let cert = params.signed_by(&key_pair, &issuer)?;
        let key = key_pair.serialize_der();
        let der = cert.der();
        let key = PrivatePkcs8KeyDer::from(key);
        let pair = Arc::new((CertificateDer::from(der.clone()), PrivateKeyDer::from(key)));
        self.cert_cache.insert(domain.to_string(), pair.clone());
        Ok(pair)
    }

    pub fn export_ca_cert(&self, output: &Path) -> Result<()> {
        std::fs::write(output, &self.ca_pem)?;
        Ok(())
    }

    pub fn ca_pem(&self) -> Result<String> {
        Ok(self.ca_pem.clone())
    }

    pub fn ca_directory(&self) -> &Path {
        &self.ca_dir
    }
}
