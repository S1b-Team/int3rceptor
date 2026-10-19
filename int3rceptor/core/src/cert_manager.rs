use crate::error::{ProxyError, Result};
use dashmap::DashMap;
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose, PublicKeyData,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

const CERT_FILE: &str = "ca_cert.pem";
const KEY_FILE: &str = "ca_key.pem";
/// Lock file held while the CA is being created, so concurrent startups
/// (or parallel tests) sharing a directory cannot interleave their writes.
const LOCK_FILE: &str = "ca.lock";

/// Leaf certificates start a little in the past to tolerate clock skew
/// between this machine and the client.
const LEAF_BACKDATE: Duration = Duration::days(1);
/// Leaf lifetime. Apple platforms reject leaves from user-installed roots that
/// are valid for more than 825 days, and browsers cap publicly trusted leaves
/// at 398 days, so stay under both.
const LEAF_VALIDITY: Duration = Duration::days(397);

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
    ///
    /// Creation is atomic: the key and certificate are written to temporary
    /// files and renamed into place while holding `ca.lock`, so another
    /// process never sees (or produces) a half-written or mismatched pair.
    /// An existing CA is only ever loaded, never overwritten.
    pub fn with_dir(ca_dir: PathBuf) -> Result<Self> {
        if !ca_dir.exists() {
            crate::storage::create_private_dir_all(&ca_dir)?;
        }
        let ca_pem_path = ca_dir.join(CERT_FILE);
        let ca_key_path = ca_dir.join(KEY_FILE);

        let (ca_pem, ca_key_pair) = if ca_pem_path.exists() && ca_key_path.exists() {
            Self::load_existing(&ca_dir, &ca_pem_path, &ca_key_path)?
        } else {
            Self::load_or_create_locked(&ca_dir, &ca_pem_path, &ca_key_path)?
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

    /// Slow path: take the creation lock, then re-check, because another
    /// process may have finished creating the CA while we waited.
    fn load_or_create_locked(
        ca_dir: &Path,
        cert_path: &Path,
        key_path: &Path,
    ) -> Result<(String, KeyPair)> {
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(ca_dir.join(LOCK_FILE))?;
        // Released when `lock` is dropped, including if this process dies.
        lock.lock()?;

        match (cert_path.exists(), key_path.exists()) {
            (true, true) => Self::load_existing(ca_dir, cert_path, key_path),
            (false, false) => Self::create_and_publish(cert_path, key_path),
            _ => Err(ProxyError::CertificateVerification(format!(
                "incomplete CA in {dir}: expected both {CERT_FILE} and {KEY_FILE}, found only one. \
                 Refusing to generate a new CA over it. To reset, delete {dir} and restart, then \
                 re-install the new CA certificate wherever the old one was trusted.",
                dir = ca_dir.display()
            ))),
        }
    }

    /// Generate a CA and move it into place. Must be called with the lock held
    /// and only when neither file exists.
    fn create_and_publish(cert_path: &Path, key_path: &Path) -> Result<(String, KeyPair)> {
        let (_params, cert, key_pair) = Self::create_ca()?;
        let pem = cert.pem();
        // Key first: if we die between the two renames, what is left behind is
        // a key nobody trusts, never a certificate without its key.
        write_atomic(key_path, key_pair.serialize_pem().as_bytes(), true)?;
        write_atomic(cert_path, pem.as_bytes(), false)?;
        if let Some(dir) = cert_path.parent() {
            sync_dir(dir);
        }
        Ok((pem, key_pair))
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

    /// Load the saved CA and fail closed unless the key belongs to the certificate.
    ///
    /// A mismatch is never repaired by generating a new CA: the user may have
    /// trusted the existing certificate, and silently swapping it would break
    /// HTTPS interception in a confusing way.
    fn load_existing(
        ca_dir: &Path,
        cert_path: &Path,
        key_path: &Path,
    ) -> Result<(String, KeyPair)> {
        restrict_key_permissions(key_path)?;
        let pem = std::fs::read_to_string(cert_path)?;
        let key_pem = std::fs::read_to_string(key_path)?;
        let key_pair = KeyPair::from_pem(&key_pem)?;

        let cert_public_key = ca_public_key(&pem).map_err(|reason| {
            ProxyError::CertificateVerification(format!(
                "cannot read CA certificate {}: {reason}",
                cert_path.display()
            ))
        })?;
        if cert_public_key != key_pair.der_bytes() {
            return Err(ProxyError::CertificateVerification(format!(
                "CA private key {key} does not match CA certificate {cert}. Refusing to start \
                 rather than replace a CA you may already trust. To reset, delete {dir} and \
                 restart; a new CA will be created. Then remove the old Interceptor CA from your \
                 browser/OS trust store and install the new one (interceptor --export-ca <file>).",
                key = key_path.display(),
                cert = cert_path.display(),
                dir = ca_dir.display(),
            )));
        }

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
        let now = OffsetDateTime::now_utc();
        params.not_before = now - LEAF_BACKDATE;
        params.not_after = now + LEAF_VALIDITY;
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
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

/// Raw subject public key bytes of the first certificate in `pem`.
fn ca_public_key(pem: &str) -> std::result::Result<Vec<u8>, String> {
    let (_, block) =
        x509_parser::pem::parse_x509_pem(pem.as_bytes()).map_err(|e| format!("bad PEM: {e}"))?;
    let cert = block
        .parse_x509()
        .map_err(|e| format!("bad certificate: {e}"))?;
    Ok(cert
        .tbs_certificate
        .subject_pki
        .subject_public_key
        .data
        .to_vec())
}

/// Write `contents` to a fresh temporary file next to `path`, fsync it, and
/// rename it over `path`. With `private`, the file is created 0600 on Unix.
fn write_atomic(path: &Path, contents: &[u8], private: bool) -> std::io::Result<()> {
    use std::io::Write;
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("ca");
    let tmp = dir.join(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(if private { 0o600 } else { 0o644 });
    }
    #[cfg(not(unix))]
    let _ = private;

    let result = (|| {
        let mut file = opts.open(&tmp)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Best-effort fsync of a directory so completed renames survive a crash.
fn sync_dir(dir: &Path) {
    #[cfg(unix)]
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
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

#[cfg(test)]
mod tests {
    use super::*;
    use x509_parser::prelude::{FromDer, X509Certificate};

    #[test]
    fn test_mismatched_key_fails_closed_without_regenerating() {
        let tmp = tempfile::tempdir().unwrap();
        let ca_dir = tmp.path().join("ca");
        let other_dir = tmp.path().join("other");
        CertManager::with_dir(ca_dir.clone()).unwrap();
        CertManager::with_dir(other_dir.clone()).unwrap();

        let cert_before = std::fs::read_to_string(ca_dir.join(CERT_FILE)).unwrap();
        std::fs::copy(other_dir.join(KEY_FILE), ca_dir.join(KEY_FILE)).unwrap();
        let key_before = std::fs::read_to_string(ca_dir.join(KEY_FILE)).unwrap();

        let err = match CertManager::with_dir(ca_dir.clone()) {
            Ok(_) => panic!("mismatched CA key was accepted"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("does not match"), "unexpected error: {err}");
        assert!(
            err.contains("delete"),
            "error should explain how to reset: {err}"
        );

        // Nothing on disk was replaced.
        assert_eq!(
            std::fs::read_to_string(ca_dir.join(CERT_FILE)).unwrap(),
            cert_before
        );
        assert_eq!(
            std::fs::read_to_string(ca_dir.join(KEY_FILE)).unwrap(),
            key_before
        );
    }

    #[test]
    fn test_incomplete_ca_fails_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let ca_dir = tmp.path().join("ca");
        CertManager::with_dir(ca_dir.clone()).unwrap();
        std::fs::remove_file(ca_dir.join(KEY_FILE)).unwrap();

        assert!(CertManager::with_dir(ca_dir.clone()).is_err());
        assert!(!ca_dir.join(KEY_FILE).exists(), "a new key was generated");
    }

    #[test]
    fn test_concurrent_creation_yields_one_consistent_ca() {
        let tmp = tempfile::tempdir().unwrap();
        let ca_dir = tmp.path().join("ca");

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let dir = ca_dir.clone();
                std::thread::spawn(move || CertManager::with_dir(dir).unwrap().ca_pem().unwrap())
            })
            .collect();
        let pems: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert!(
            pems.iter().all(|pem| pem == &pems[0]),
            "callers saw different CAs"
        );
        // The pair on disk loads (and therefore matches) and is the one everyone got.
        let reloaded = CertManager::with_dir(ca_dir).unwrap().ca_pem().unwrap();
        assert_eq!(reloaded, pems[0]);
    }

    #[test]
    fn test_leaf_validity_and_server_auth() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = CertManager::with_dir(tmp.path().join("ca")).unwrap();
        let pair = mgr.generate_cert("leaf.example.test").unwrap();
        let (_, leaf) = X509Certificate::from_der(pair.0.as_ref()).unwrap();

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let not_before = leaf.validity().not_before.timestamp();
        let not_after = leaf.validity().not_after.timestamp();
        assert!(not_before < now, "leaf should start in the past");
        assert!(now - not_before <= 2 * 86_400, "leaf backdated too far");
        assert!(not_after > now);
        assert!(
            not_after - not_before <= 825 * 86_400,
            "leaf valid for more than 825 days"
        );

        let eku = leaf
            .extended_key_usage()
            .unwrap()
            .expect("leaf has no extended key usage");
        assert!(eku.value.server_auth);

        // Still chains to the CA.
        let ca_pem = mgr.ca_pem().unwrap();
        let (_, ca_block) = x509_parser::pem::parse_x509_pem(ca_pem.as_bytes()).unwrap();
        let ca = ca_block.parse_x509().unwrap();
        leaf.verify_signature(Some(ca.public_key())).unwrap();
    }
}
