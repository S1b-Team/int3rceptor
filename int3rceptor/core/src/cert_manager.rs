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
        Self::with_dir(Self::default_ca_dir())
    }

    /// Load or create the CA in a specific directory.
    ///
    /// On Unix a newly created directory gets mode 0700 and the private key is
    /// written with mode 0600. An existing key file that is readable by group
    /// or others is tightened to 0600.
    pub fn with_dir(ca_dir: PathBuf) -> Result<Self> {
        if !ca_dir.exists() {
            crate::storage::create_private_dir_all(&ca_dir)?;
        }
        let ca_pem_path = ca_dir.join(CERT_FILE);
        let ca_key_path = ca_dir.join(KEY_FILE);

        let (ca_pem, ca_key_pair) = if ca_pem_path.exists() && ca_key_path.exists() {
            restrict_key_permissions(&ca_key_path)?;
            Self::load_existing(&ca_pem_path, &ca_key_path)?
        } else {
            let (_params, cert, key_pair) = Self::create_ca()?;
            let pem = cert.pem();
            std::fs::write(&ca_pem_path, &pem)?;
            write_private_file(&ca_key_path, key_pair.serialize_pem().as_bytes())?;
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

/// Write a file that only the owner can read (0600 on Unix).
fn write_private_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    // `mode` only applies on creation; enforce it for pre-existing files too.
    restrict_key_permissions(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

/// Tighten an existing private key file to 0600 if group/other can access it.
fn restrict_key_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode();
        if mode & 0o077 != 0 {
            tracing::warn!(path = %path.display(), "CA private key was accessible by other users; restricting to 0600");
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(all(test, unix))]
mod permission_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(p: &Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn test_new_ca_dir_and_key_are_private() {
        let tmp = tempfile::tempdir().unwrap();
        let ca_dir = tmp.path().join("ca");
        let mgr = CertManager::with_dir(ca_dir.clone()).unwrap();
        assert_eq!(mode(&ca_dir), 0o700);
        assert_eq!(mode(&ca_dir.join(KEY_FILE)), 0o600);
        assert_eq!(mgr.ca_directory(), ca_dir.as_path());
    }

    #[test]
    fn test_existing_loose_key_is_tightened_and_ca_reused() {
        let tmp = tempfile::tempdir().unwrap();
        let ca_dir = tmp.path().join("ca");
        let first = CertManager::with_dir(ca_dir.clone())
            .unwrap()
            .ca_pem()
            .unwrap();
        let key = ca_dir.join(KEY_FILE);
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o644)).unwrap();
        let second = CertManager::with_dir(ca_dir.clone())
            .unwrap()
            .ca_pem()
            .unwrap();
        assert_eq!(mode(&key), 0o600);
        assert_eq!(first, second);
    }
}
