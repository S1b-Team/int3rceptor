//! Import of HAR 1.2 files into capture entries.
//!
//! Two header layouts are accepted: the standard `[{"name": .., "value": ..}]`
//! written by browsers and proxies, and the `[[name, value], ..]` pairs written by
//! this project's own HAR export. Entries that cannot be used are skipped and
//! counted, so one bad entry does not discard the rest of a large file.

use crate::capture::{CaptureEntry, CapturedRequest, CapturedResponse};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::Value;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Upper bound on entries per import. The request body limit of the API bounds the
/// size; this bounds the work done per call.
pub const MAX_HAR_ENTRIES: usize = 10_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HarError {
    #[error("not valid JSON: {0}")]
    InvalidJson(String),
    #[error("not a HAR file: missing log.entries array")]
    MissingEntries,
    #[error("too many entries: {count} (maximum {max})")]
    TooManyEntries { count: usize, max: usize },
}

/// Result of parsing a HAR file.
#[derive(Debug)]
pub struct HarImport {
    /// Usable entries, oldest first. Ids are assigned when they are captured.
    pub entries: Vec<CaptureEntry>,
    /// Entries that were present but could not be used.
    pub skipped: usize,
}

/// Parse a HAR document. Never makes network requests.
pub fn parse_har(bytes: &[u8]) -> Result<HarImport, HarError> {
    let root: Value =
        serde_json::from_slice(bytes).map_err(|e| HarError::InvalidJson(e.to_string()))?;
    let raw = root
        .pointer("/log/entries")
        .and_then(Value::as_array)
        .ok_or(HarError::MissingEntries)?;
    if raw.len() > MAX_HAR_ENTRIES {
        return Err(HarError::TooManyEntries {
            count: raw.len(),
            max: MAX_HAR_ENTRIES,
        });
    }

    let mut entries: Vec<CaptureEntry> = raw.iter().filter_map(parse_entry).collect();
    let skipped = raw.len() - entries.len();
    // Capture shows newest first, so push oldest first. The sort is stable.
    entries.sort_by_key(|entry| entry.request.timestamp_ms);
    Ok(HarImport { entries, skipped })
}

fn parse_entry(entry: &Value) -> Option<CaptureEntry> {
    let request = entry.get("request")?;

    let method = request.get("method")?.as_str()?;
    if method.is_empty() || !method.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    let url = request.get("url")?.as_str()?;
    let tls = is_http_url(url)?;

    let mut captured = CapturedRequest::new(method.to_ascii_uppercase(), url, tls);
    captured.headers = parse_headers(request.get("headers"));
    captured.body = request
        .pointer("/postData/text")
        .and_then(Value::as_str)
        .map(|text| text.as_bytes().to_vec())
        .unwrap_or_default();
    if let Some(ms) = entry
        .get("startedDateTime")
        .and_then(Value::as_str)
        .and_then(parse_timestamp_ms)
    {
        captured.timestamp_ms = ms;
    }

    let response = entry
        .get("response")
        .and_then(|resp| parse_response(resp, entry.get("time")));
    Some(CaptureEntry {
        request: captured,
        response,
    })
}

/// `Some(true)` for https, `Some(false)` for http, `None` for anything else.
fn is_http_url(url: &str) -> Option<bool> {
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") {
        Some(true)
    } else if lower.starts_with("http://") {
        Some(false)
    } else {
        None
    }
}

fn parse_response(response: &Value, time_ms: Option<&Value>) -> Option<CapturedResponse> {
    let status = response.get("status")?.as_u64()?;
    // A status of 0 is how HAR (and this project's export) mark "no response".
    if status == 0 || status > 999 {
        return None;
    }

    let content = response.get("content");
    let text = content
        .and_then(|c| c.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let is_base64 = content
        .and_then(|c| c.get("encoding"))
        .and_then(Value::as_str)
        .is_some_and(|e| e.eq_ignore_ascii_case("base64"));
    let body = if is_base64 {
        BASE64.decode(text).unwrap_or_default()
    } else {
        text.as_bytes().to_vec()
    };

    // HAR uses -1 for "unknown".
    let duration_ms = time_ms
        .and_then(Value::as_f64)
        .filter(|ms| ms.is_finite() && *ms > 0.0)
        .map_or(0, |ms| ms.round() as u128);

    Some(CapturedResponse {
        request_id: 0,
        status_code: status as u16,
        headers: parse_headers(response.get("headers")),
        body,
        duration_ms,
    })
}

fn parse_headers(value: Option<&Value>) -> Vec<(String, String)> {
    let Some(items) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let (name, value) = match item {
                Value::Object(map) => (map.get("name")?.as_str()?, map.get("value")?.as_str()?),
                Value::Array(pair) if pair.len() == 2 => (pair[0].as_str()?, pair[1].as_str()?),
                _ => return None,
            };
            // HTTP/2 pseudo-headers (":authority") are not valid to send again.
            if name.is_empty() || name.starts_with(':') {
                return None;
            }
            Some((name.to_string(), value.to_string()))
        })
        .collect()
}

fn parse_timestamp_ms(value: &str) -> Option<i128> {
    OffsetDateTime::parse(value, &Rfc3339)
        .ok()
        .map(|dt| dt.unix_timestamp_nanos() / 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn har(entries: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"log": {"version": "1.2", "entries": entries}})).unwrap()
    }

    fn browser_entry() -> Value {
        json!({
            "startedDateTime": "2026-03-01T10:00:00.250Z",
            "time": 123.4,
            "request": {
                "method": "post",
                "url": "https://example.com/api/login",
                "headers": [
                    {"name": ":authority", "value": "example.com"},
                    {"name": "Content-Type", "value": "application/json"}
                ],
                "postData": {"mimeType": "application/json", "text": "{\"u\":1}"}
            },
            "response": {
                "status": 401,
                "headers": [{"name": "Content-Type", "value": "text/plain"}],
                "content": {"size": 4, "text": "nope"}
            }
        })
    }

    #[test]
    fn imports_a_browser_style_entry() {
        let import = parse_har(&har(json!([browser_entry()]))).unwrap();
        assert_eq!(import.skipped, 0);
        let entry = &import.entries[0];

        assert_eq!(entry.request.method, "POST");
        assert_eq!(entry.request.url, "https://example.com/api/login");
        assert!(entry.request.tls);
        assert_eq!(entry.request.body, b"{\"u\":1}");
        assert_eq!(entry.request.timestamp_ms, 1_772_359_200_250);
        // The pseudo-header is dropped, the real one is kept.
        assert_eq!(
            entry.request.headers,
            vec![("Content-Type".to_string(), "application/json".to_string())]
        );

        let response = entry.response.as_ref().unwrap();
        assert_eq!(response.status_code, 401);
        assert_eq!(response.body, b"nope");
        assert_eq!(response.duration_ms, 123);
    }

    #[test]
    fn accepts_the_pair_headers_written_by_our_own_export() {
        let entry = json!({
            "startedDateTime": "2026-03-01T10:00:00Z",
            "time": 5,
            "request": {
                "method": "GET",
                "url": "http://example.com/",
                "headers": [["Host", "example.com"], ["Accept", "*/*"]],
                "bodySize": 0
            },
            "response": {"status": 200, "headers": [["Server", "x"]], "bodySize": 0}
        });
        let import = parse_har(&har(json!([entry]))).unwrap();
        let entry = &import.entries[0];
        assert!(!entry.request.tls);
        assert_eq!(entry.request.headers.len(), 2);
        assert_eq!(
            entry.response.as_ref().unwrap().headers,
            vec![("Server".to_string(), "x".to_string())]
        );
    }

    #[test]
    fn decodes_base64_response_bodies() {
        let mut entry = browser_entry();
        entry["response"]["content"] = json!({"encoding": "base64", "text": "AAEC/w=="});
        let import = parse_har(&har(json!([entry]))).unwrap();
        let response = import.entries[0].response.as_ref().unwrap();
        assert_eq!(response.body, vec![0, 1, 2, 255]);
    }

    #[test]
    fn status_zero_means_no_response() {
        let mut entry = browser_entry();
        entry["response"]["status"] = json!(0);
        let import = parse_har(&har(json!([entry]))).unwrap();
        assert!(import.entries[0].response.is_none());
    }

    #[test]
    fn unknown_duration_is_zero_not_negative() {
        let mut entry = browser_entry();
        entry["time"] = json!(-1);
        let import = parse_har(&har(json!([entry]))).unwrap();
        assert_eq!(import.entries[0].response.as_ref().unwrap().duration_ms, 0);
    }

    #[test]
    fn unusable_entries_are_skipped_and_counted() {
        let entries = json!([
            browser_entry(),
            {"request": {"method": "GET", "url": "ftp://example.com/file"}},
            {"request": {"method": "GET", "url": "http://exa mple.com/"}},
            {"request": {"method": "G3T", "url": "http://example.com/"}},
            {"request": {"url": "http://example.com/"}},
            {"nothing": true},
            "not an object"
        ]);
        let import = parse_har(&har(entries)).unwrap();
        assert_eq!(import.entries.len(), 1);
        assert_eq!(import.skipped, 6);
    }

    #[test]
    fn entries_come_out_oldest_first() {
        let mut newer = browser_entry();
        newer["startedDateTime"] = json!("2026-03-01T12:00:00Z");
        let mut older = browser_entry();
        older["startedDateTime"] = json!("2026-03-01T08:00:00Z");
        let import = parse_har(&har(json!([newer, older]))).unwrap();
        assert!(import.entries[0].request.timestamp_ms < import.entries[1].request.timestamp_ms);
    }

    #[test]
    fn rejects_input_that_is_not_a_har_file() {
        assert!(matches!(
            parse_har(b"not json"),
            Err(HarError::InvalidJson(_))
        ));
        assert_eq!(parse_har(b"{}").unwrap_err(), HarError::MissingEntries);
        assert_eq!(
            parse_har(br#"{"log": {"entries": "nope"}}"#).unwrap_err(),
            HarError::MissingEntries
        );
        assert_eq!(parse_har(b"[]").unwrap_err(), HarError::MissingEntries);
    }

    #[test]
    fn rejects_files_with_too_many_entries() {
        let entries: Vec<Value> = (0..=MAX_HAR_ENTRIES).map(|_| json!({})).collect();
        let err = parse_har(&har(Value::Array(entries))).unwrap_err();
        assert_eq!(
            err,
            HarError::TooManyEntries {
                count: MAX_HAR_ENTRIES + 1,
                max: MAX_HAR_ENTRIES
            }
        );
    }

    #[test]
    fn an_empty_entries_array_is_a_valid_empty_import() {
        let import = parse_har(&har(json!([]))).unwrap();
        assert!(import.entries.is_empty());
        assert_eq!(import.skipped, 0);
    }
}
