use crate::capture::{CaptureEntry, CaptureQuery, CapturedRequest, CapturedResponse};
use crate::database::EncryptionKeyProvider;
use crate::error::Result;
use rusqlite::{params, Connection, OpenFlags, ToSql};
use serde_json;
use std::cmp::min;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug)]
pub struct CaptureStorage {
    path: PathBuf,
    encryption: EncryptionKeyProvider,
}

/// Header added to a decoded entry when one of its encrypted fields could
/// not be decrypted or parsed. The value names the affected field(s).
pub const DECRYPT_ERROR_HEADER: &str = "x-interceptor-decrypt-error";

impl CaptureStorage {
    /// Open storage using the key configured in the environment.
    /// Fails if no valid key is configured and plaintext was not explicitly enabled.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        Self::with_encryption(path, EncryptionKeyProvider::new()?)
    }

    /// Open storage with encryption explicitly disabled (tests only).
    ///
    /// This always stores plaintext and never reads the environment.
    pub fn new_unencrypted(path: impl AsRef<Path>) -> Result<Self> {
        Self::with_encryption(path, EncryptionKeyProvider::disabled())
    }

    /// Open storage with an explicit key provider.
    pub fn with_encryption(
        path: impl AsRef<Path>,
        encryption: EncryptionKeyProvider,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        prepare_db_path(&path)?;
        let storage = Self { path, encryption };
        storage.init()?;
        Ok(storage)
    }

    fn connect(&self) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        conn.busy_timeout(Duration::from_secs(1))?;
        Ok(conn)
    }

    fn init(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS captures (
                id INTEGER PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                headers TEXT NOT NULL,
                body BLOB,
                tls INTEGER NOT NULL,
                resp_status INTEGER,
                resp_headers TEXT,
                resp_body BLOB,
                duration_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_captures_method ON captures(method);
            CREATE INDEX IF NOT EXISTS idx_captures_url ON captures(url);
            CREATE INDEX IF NOT EXISTS idx_captures_status ON captures(resp_status);
            "#,
        )?;
        Ok(())
    }

    pub fn insert(&self, entry: &CaptureEntry) -> Result<()> {
        use crate::database;

        let conn = self.connect()?;

        // Encrypt sensitive request data
        let headers_json = serde_json::to_string(&entry.request.headers)?;
        let encrypted_headers =
            database::encrypt_if_enabled(&self.encryption, headers_json.as_bytes())?;

        let encrypted_body = if entry.request.body.is_empty() {
            None
        } else {
            Some(database::encrypt_if_enabled(
                &self.encryption,
                &entry.request.body,
            )?)
        };

        // Encrypt sensitive response data
        let resp_headers = entry
            .response
            .as_ref()
            .map(|r| {
                let json = serde_json::to_string(&r.headers)?;
                database::encrypt_if_enabled(&self.encryption, json.as_bytes())
            })
            .transpose()?;

        let resp_body = entry.response.as_ref().and_then(|r| {
            if r.body.is_empty() {
                None
            } else {
                Some(r.body.as_slice())
            }
        });

        let encrypted_resp_body = resp_body
            .map(|b| database::encrypt_if_enabled(&self.encryption, b))
            .transpose()?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO captures (
                id,
                timestamp_ms,
                method,
                url,
                headers,
                body,
                tls,
                resp_status,
                resp_headers,
                resp_body,
                duration_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                entry.request.id as i64,
                clamp_i128(entry.request.timestamp_ms),
                entry.request.method,
                entry.request.url,
                encrypted_headers,
                encrypted_body,
                if entry.request.tls { 1 } else { 0 },
                entry.response.as_ref().map(|r| r.status_code as i64),
                resp_headers,
                encrypted_resp_body,
                entry.response.as_ref().map(|r| clamp_u128(r.duration_ms)),
            ],
        )?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM captures", [])?;
        Ok(())
    }

    pub fn query(&self, filter: &CaptureQuery) -> Result<Vec<CaptureEntry>> {
        let mut sql = String::from(
            "SELECT id, timestamp_ms, method, url, headers, body, tls, resp_status, resp_headers, resp_body, duration_ms FROM captures WHERE 1=1",
        );
        let mut values: Vec<String> = Vec::new();

        if let Some(method) = &filter.method {
            sql.push_str(" AND method = ?");
            values.push(method.clone());
        }
        if let Some(host) = &filter.host {
            sql.push_str(" AND url LIKE ?");
            values.push(format!("%{}%", host));
        }
        if let Some(status) = filter.status {
            sql.push_str(" AND resp_status = ?");
            values.push(status.to_string());
        }
        if let Some(tls) = filter.tls {
            sql.push_str(" AND tls = ?");
            values.push(if tls { "1" } else { "0" }.into());
        }
        if let Some(search) = &filter.search {
            sql.push_str(" AND (url LIKE ?)"); // Note: can't search encrypted headers
            let pattern = format!("%{}%", search);
            values.push(pattern);
        }
        sql.push_str(" ORDER BY id DESC");
        let limit = filter.limit.unwrap_or(500);
        sql.push_str(&format!(" LIMIT {}", limit));

        let conn = self.connect()?;
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn ToSql> = values.iter().map(|v| v as &dyn ToSql).collect();
        let mut rows = stmt.query(&param_refs[..])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let id = row.get::<_, i64>(0)? as u64;
            let mut failed: Vec<&'static str> = Vec::new();

            // Decrypt sensitive request data
            let encrypted_headers: Vec<u8> = row.get(4)?;
            let mut headers =
                self.decode_headers(id, "request headers", Some(&encrypted_headers), &mut failed);
            let encrypted_body: Option<Vec<u8>> = row.get(5)?;
            let body = self.decode_body(id, "request body", encrypted_body.as_deref(), &mut failed);

            let response = match row.get::<_, Option<i64>>(7)? {
                Some(status) => {
                    let encrypted_resp_headers: Option<Vec<u8>> = row.get(8)?;
                    let headers = self.decode_headers(
                        id,
                        "response headers",
                        encrypted_resp_headers.as_deref(),
                        &mut failed,
                    );
                    let encrypted_resp_body: Option<Vec<u8>> = row.get(9)?;
                    let body = self.decode_body(
                        id,
                        "response body",
                        encrypted_resp_body.as_deref(),
                        &mut failed,
                    );
                    let duration_ms = row.get::<_, Option<i64>>(10)?.unwrap_or(0) as u128;
                    Some(CapturedResponse {
                        request_id: id,
                        status_code: status as u16,
                        headers,
                        body,
                        duration_ms,
                    })
                }
                None => None,
            };

            // Make unreadable fields visible instead of silently empty.
            if !failed.is_empty() {
                headers.push((DECRYPT_ERROR_HEADER.to_string(), failed.join(", ")));
            }

            let request = CapturedRequest {
                id,
                timestamp_ms: row.get::<_, i64>(1)? as i128,
                method: row.get(2)?,
                url: row.get(3)?,
                headers,
                body,
                tls: row.get::<_, i64>(6)? == 1,
            };
            entries.push(CaptureEntry { request, response });
        }
        Ok(entries)
    }
}

impl CaptureStorage {
    fn decrypt_field(
        &self,
        id: u64,
        field: &'static str,
        data: &[u8],
        failed: &mut Vec<&'static str>,
    ) -> Option<Vec<u8>> {
        match crate::database::decrypt_if_enabled(&self.encryption, data) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!(capture_id = id, field, error = %e, "failed to decrypt stored capture field (wrong key or plaintext database?)");
                failed.push(field);
                None
            }
        }
    }

    fn decode_headers(
        &self,
        id: u64,
        field: &'static str,
        data: Option<&[u8]>,
        failed: &mut Vec<&'static str>,
    ) -> Vec<(String, String)> {
        let Some(data) = data else { return Vec::new() };
        let Some(plain) = self.decrypt_field(id, field, data, failed) else {
            return Vec::new();
        };
        match serde_json::from_slice::<Vec<(String, String)>>(&plain) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(capture_id = id, field, error = %e, "stored capture headers are not valid JSON");
                failed.push(field);
                Vec::new()
            }
        }
    }

    fn decode_body(
        &self,
        id: u64,
        field: &'static str,
        data: Option<&[u8]>,
        failed: &mut Vec<&'static str>,
    ) -> Vec<u8> {
        data.and_then(|d| self.decrypt_field(id, field, d, failed))
            .unwrap_or_default()
    }
}

/// Create the database directory (0700) and file (0600) with restrictive
/// permissions on Unix before SQLite opens it. Existing paths are left alone.
fn prepare_db_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            create_private_dir_all(parent)?;
        }
    }
    if !path.exists() {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// `create_dir_all` that uses mode 0700 for newly created directories on Unix.
pub(crate) fn create_private_dir_all(dir: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

fn clamp_i128(value: i128) -> i64 {
    value.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

fn clamp_u128(value: u128) -> i64 {
    min(value, i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_entry(id: u64, method: &str, url: &str, status: Option<u16>) -> CaptureEntry {
        let request = CapturedRequest {
            id,
            timestamp_ms: 1700000000000,
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![("host".to_string(), "test.com".to_string())],
            body: b"request body".to_vec(),
            tls: url.starts_with("https"),
        };

        let response = status.map(|s| CapturedResponse {
            request_id: id,
            status_code: s,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: b"response body".to_vec(),
            duration_ms: 100,
        });

        CaptureEntry { request, response }
    }

    #[test]
    fn test_storage_create_and_init() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let storage = CaptureStorage::new_unencrypted(&db_path);
        assert!(storage.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn test_storage_insert_and_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        let entry = create_test_entry(1, "GET", "https://api.test.com/users", Some(200));
        storage.insert(&entry).unwrap();

        let results = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request.method, "GET");
        assert_eq!(results[0].request.url, "https://api.test.com/users");
        assert_eq!(results[0].response.as_ref().unwrap().status_code, 200);
    }

    #[test]
    fn test_storage_query_by_method() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(1, "GET", "/api/users", Some(200)))
            .unwrap();
        storage
            .insert(&create_test_entry(2, "POST", "/api/users", Some(201)))
            .unwrap();
        storage
            .insert(&create_test_entry(3, "GET", "/api/posts", Some(200)))
            .unwrap();

        let query = CaptureQuery {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.request.method == "GET"));
    }

    #[test]
    fn test_storage_query_by_host() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(
                1,
                "GET",
                "https://api.example.com/users",
                Some(200),
            ))
            .unwrap();
        storage
            .insert(&create_test_entry(
                2,
                "GET",
                "https://other.com/data",
                Some(200),
            ))
            .unwrap();

        let query = CaptureQuery {
            host: Some("example.com".to_string()),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].request.url.contains("example.com"));
    }

    #[test]
    fn test_storage_query_by_status() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(1, "GET", "/success", Some(200)))
            .unwrap();
        storage
            .insert(&create_test_entry(2, "GET", "/error", Some(500)))
            .unwrap();
        storage
            .insert(&create_test_entry(3, "GET", "/not-found", Some(404)))
            .unwrap();

        let query = CaptureQuery {
            status: Some(200),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].response.as_ref().unwrap().status_code, 200);
    }

    #[test]
    fn test_storage_query_by_tls() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(
                1,
                "GET",
                "https://secure.com",
                Some(200),
            ))
            .unwrap();
        storage
            .insert(&create_test_entry(
                2,
                "GET",
                "http://insecure.com",
                Some(200),
            ))
            .unwrap();

        let query = CaptureQuery {
            tls: Some(true),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].request.tls);
    }

    #[test]
    fn test_storage_query_with_search() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(1, "GET", "/api/v1/users", Some(200)))
            .unwrap();
        storage
            .insert(&create_test_entry(2, "GET", "/web/home", Some(200)))
            .unwrap();

        let query = CaptureQuery {
            search: Some("/api/".to_string()),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].request.url.contains("/api/"));
    }

    #[test]
    fn test_storage_query_with_limit() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        for i in 0..10 {
            storage
                .insert(&create_test_entry(
                    i,
                    "GET",
                    &format!("/page/{}", i),
                    Some(200),
                ))
                .unwrap();
        }

        let query = CaptureQuery {
            limit: Some(3),
            ..Default::default()
        };

        let results = storage.query(&query).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_storage_clear() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        storage
            .insert(&create_test_entry(1, "GET", "/test1", Some(200)))
            .unwrap();
        storage
            .insert(&create_test_entry(2, "GET", "/test2", Some(200)))
            .unwrap();

        let before = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(before.len(), 2);

        storage.clear().unwrap();

        let after = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(after.len(), 0);
    }

    #[test]
    fn test_storage_entry_without_response() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        let entry = create_test_entry(1, "GET", "/pending", None);
        storage.insert(&entry).unwrap();

        let results = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].response.is_none());
    }

    #[test]
    fn test_storage_preserves_body_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        let entry = create_test_entry(1, "POST", "/data", Some(200));
        storage.insert(&entry).unwrap();

        let results = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(results[0].request.body, b"request body");
        assert_eq!(results[0].response.as_ref().unwrap().body, b"response body");
    }

    #[test]
    fn test_storage_upsert_same_id() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();

        // Insert with same ID twice (should replace)
        storage
            .insert(&create_test_entry(1, "GET", "/original", Some(200)))
            .unwrap();
        storage
            .insert(&create_test_entry(1, "GET", "/updated", Some(201)))
            .unwrap();

        let results = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request.url, "/updated");
        assert_eq!(results[0].response.as_ref().unwrap().status_code, 201);
    }

    #[test]
    fn test_clamp_functions() {
        // Test clamp_i128
        assert_eq!(clamp_i128(100), 100);
        assert_eq!(clamp_i128(i64::MAX as i128 + 1), i64::MAX);
        assert_eq!(clamp_i128(i64::MIN as i128 - 1), i64::MIN);

        // Test clamp_u128
        assert_eq!(clamp_u128(100), 100);
        assert_eq!(clamp_u128(i64::MAX as u128 + 1), i64::MAX);
    }

    // Test fixtures only: fixed, obviously fake keys.
    fn test_key(byte: u8) -> EncryptionKeyProvider {
        EncryptionKeyProvider::with_key([byte; 32])
    }

    #[test]
    fn test_encrypted_roundtrip_and_ciphertext_on_disk() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("enc.db");
        let storage = CaptureStorage::with_encryption(&db_path, test_key(7)).unwrap();
        storage
            .insert(&create_test_entry(1, "POST", "https://x.test/a", Some(200)))
            .unwrap();
        let got = storage.query(&CaptureQuery::default()).unwrap();
        assert_eq!(got[0].request.body, b"request body");
        assert!(!got[0]
            .request
            .headers
            .iter()
            .any(|(k, _)| k == DECRYPT_ERROR_HEADER));
        let raw = std::fs::read(&db_path).unwrap();
        assert!(!raw.windows(12).any(|w| w == b"request body"));
    }

    #[test]
    fn test_wrong_key_marks_entry_instead_of_silently_empty() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("enc.db");
        CaptureStorage::with_encryption(&db_path, test_key(1))
            .unwrap()
            .insert(&create_test_entry(1, "GET", "https://x.test/", Some(200)))
            .unwrap();
        let got = CaptureStorage::with_encryption(&db_path, test_key(2))
            .unwrap()
            .query(&CaptureQuery::default())
            .unwrap();
        let marker = got[0]
            .request
            .headers
            .iter()
            .find(|(k, _)| k == DECRYPT_ERROR_HEADER)
            .expect("decrypt failure must be marked");
        assert!(marker.1.contains("request headers"));
        assert!(marker.1.contains("response body"));
    }

    #[test]
    fn test_new_unencrypted_is_always_plaintext() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("plain.db");
        let storage = CaptureStorage::new_unencrypted(&db_path).unwrap();
        storage
            .insert(&create_test_entry(1, "GET", "http://x.test/", None))
            .unwrap();
        let raw = std::fs::read(&db_path).unwrap();
        assert!(raw.windows(12).any(|w| w == b"request body"));
    }

    #[cfg(unix)]
    #[test]
    fn test_db_file_and_dir_created_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let sub = dir.path().join("nested/data");
        let db_path = sub.join("c.db");
        CaptureStorage::new_unencrypted(&db_path).unwrap();
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&db_path), 0o600);
        assert_eq!(mode(&sub), 0o700);
    }
}
