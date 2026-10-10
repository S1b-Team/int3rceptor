use crate::connection_pool::{ConnectionPool, ProxyBody};
use anyhow::Result;
use http_body_util::BodyExt;
use hyper::{Method, Request, Uri};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderConfig {
    pub positions: Vec<IntruderPosition>,
    pub payloads: Vec<String>,
    pub attack_type: AttackType,
    #[serde(default)]
    pub options: IntruderOptions,
}

/// Default per-request timeout when `IntruderOptions::timeout_ms` is 0 or omitted.
pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;

fn default_timeout_ms() -> u64 {
    DEFAULT_REQUEST_TIMEOUT_MS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderOptions {
    pub concurrency: usize,
    pub delay_ms: u64,
    /// Cap how long each probe waits for headers + body. Stop aborts in-flight
    /// tasks; this timeout still covers hung targets when an attack is not stopped.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

impl Default for IntruderOptions {
    fn default() -> Self {
        Self {
            concurrency: 1,
            delay_ms: 0,
            timeout_ms: DEFAULT_REQUEST_TIMEOUT_MS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderPosition {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    Sniper,      // One payload set, iterate through each position
    Battering,   // One payload set, same payload in all positions
    Pitchfork,   // Multiple payload sets, iterate in parallel
    ClusterBomb, // Multiple payload sets, all combinations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderResult {
    pub request_id: usize,
    pub payload: String,
    pub status_code: u16,
    pub response_length: usize,
    pub duration_ms: u64,
}

/// Snapshot of an attack: whether it is still sending and how many results
/// have landed so far. Lets an API client poll start -> status -> results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderStatus {
    pub running: bool,
    pub result_count: usize,
}

pub struct Intruder {
    results: Arc<RwLock<Vec<IntruderResult>>>,
    /// True from start until in-flight requests finish (includes drain after stop).
    is_running: Arc<AtomicBool>,
    /// Cleared by stop; launch loop checks this so status can stay running while draining.
    launch_requests: Arc<AtomicBool>,
    /// Abort handles for in-flight probes so stop can cancel hung upstream waits.
    in_flight: Arc<RwLock<Vec<tokio::task::AbortHandle>>>,
}

impl Intruder {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            launch_requests: Arc::new(AtomicBool::new(false)),
            in_flight: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_attack(
        &self,
        template: String,
        config: IntruderConfig,
        pool: ConnectionPool,
    ) -> Result<()> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Attack already running"));
        }

        self.launch_requests.store(true, Ordering::SeqCst);
        self.in_flight.write().clear();
        self.clear_results();
        let requests = self.generate_requests(&template, &config)?;
        let results = self.results.clone();
        let is_running = self.is_running.clone();
        let launch_requests = self.launch_requests.clone();
        let in_flight = self.in_flight.clone();
        let concurrency = config.options.concurrency.max(1);
        let delay = config.options.delay_ms;
        let timeout_ms = if config.options.timeout_ms == 0 {
            DEFAULT_REQUEST_TIMEOUT_MS
        } else {
            config.options.timeout_ms
        };

        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
            let mut handles = Vec::new();

            for (id, (req_str, payload)) in requests.into_iter().enumerate() {
                if !launch_requests.load(Ordering::SeqCst) {
                    break;
                }

                let permit = semaphore.clone().acquire_owned().await.unwrap();
                // Re-check after the semaphore wait: stop may have landed while we queued.
                if !launch_requests.load(Ordering::SeqCst) {
                    drop(permit);
                    break;
                }

                let pool = pool.clone();
                let results = results.clone();
                let req_str = req_str.clone();
                let payload = payload.clone();
                let launch_requests = launch_requests.clone();

                if delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    if !launch_requests.load(Ordering::SeqCst) {
                        drop(permit);
                        break;
                    }
                }

                let handle = tokio::spawn(async move {
                    let _permit = permit;
                    let start = Instant::now();

                    let Ok(req) = parse_request(&req_str) else {
                        return;
                    };

                    let client = pool.client();
                    let timeout = tokio::time::Duration::from_millis(timeout_ms);
                    let outcome = tokio::time::timeout(timeout, async {
                        let resp = client.request(req).await.ok()?;
                        let status = resp.status().as_u16();
                        // Keep the status even if the body stream ends early.
                        let body_len = match resp.collect().await {
                            Ok(bytes) => bytes.to_bytes().len(),
                            Err(_) => 0,
                        };
                        Some((status, body_len))
                    })
                    .await;

                    if let Ok(Some((status, body_len))) = outcome {
                        results.write().push(IntruderResult {
                            request_id: id,
                            payload,
                            status_code: status,
                            response_length: body_len,
                            duration_ms: start.elapsed().as_millis() as u64,
                        });
                    }
                });

                in_flight.write().push(handle.abort_handle());
                handles.push(handle);
            }

            // Wait for all to finish (or abort after stop).
            for handle in handles {
                let _ = handle.await;
            }

            in_flight.write().clear();
            launch_requests.store(false, Ordering::SeqCst);
            is_running.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    pub fn stop_attack(&self) {
        // Stop launching new requests and cancel in-flight probes so a hung
        // upstream cannot leave is_running stuck true forever.
        self.launch_requests.store(false, Ordering::SeqCst);
        for handle in self.in_flight.write().drain(..) {
            handle.abort();
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Current attack state plus the number of results collected so far.
    /// `running` stays true while in-flight requests are still draining after stop.
    pub fn status(&self) -> IntruderStatus {
        IntruderStatus {
            running: self.is_running(),
            result_count: self.results.read().len(),
        }
    }

    /// Generate requests with their associated payloads
    /// Returns Vec<(request_string, payload_used)>
    pub fn generate_requests(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<(String, String)>> {
        match config.attack_type {
            AttackType::Sniper => self.generate_sniper(template, config),
            AttackType::Battering => self.generate_battering(template, config),
            AttackType::Pitchfork => self.generate_pitchfork(template, config),
            AttackType::ClusterBomb => self.generate_cluster_bomb(template, config),
        }
    }

    fn generate_sniper(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<(String, String)>> {
        let mut requests = Vec::new();

        for payload in &config.payloads {
            for position in &config.positions {
                let mut modified = template.to_string();
                let marker = format!("§{}§", position.name);
                modified = modified.replace(&marker, payload);

                // Replace other markers with empty string
                for other_pos in &config.positions {
                    if other_pos.name != position.name {
                        let other_marker = format!("§{}§", other_pos.name);
                        modified = modified.replace(&other_marker, "");
                    }
                }

                requests.push((modified, payload.clone()));
            }
        }

        Ok(requests)
    }

    fn generate_battering(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<(String, String)>> {
        let mut requests = Vec::new();

        for payload in &config.payloads {
            let mut modified = template.to_string();

            for position in &config.positions {
                let marker = format!("§{}§", position.name);
                modified = modified.replace(&marker, payload);
            }

            requests.push((modified, payload.clone()));
        }

        Ok(requests)
    }

    fn generate_pitchfork(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<(String, String)>> {
        let mut requests = Vec::new();
        let payload_count = config.payloads.len();
        let empty_string = String::new();

        for i in 0..payload_count {
            let mut modified = template.to_string();
            let mut used_payloads = Vec::new();

            for position in config.positions.iter() {
                let marker = format!("§{}§", position.name);
                let payload = config.payloads.get(i).unwrap_or(&empty_string);
                modified = modified.replace(&marker, payload);
                used_payloads.push(payload.clone());
            }

            requests.push((modified, used_payloads.join(", ")));
        }

        Ok(requests)
    }

    fn generate_cluster_bomb(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<(String, String)>> {
        let mut requests = Vec::new();
        let position_count = config.positions.len();

        if position_count == 0 {
            return Ok(requests);
        }

        // Generate all combinations
        let total_combinations = config.payloads.len().pow(position_count as u32);

        for i in 0..total_combinations {
            let mut modified = template.to_string();
            let mut combination_index = i;
            let mut used_payloads = Vec::new();

            for position in &config.positions {
                let payload_index = combination_index % config.payloads.len();
                let payload = &config.payloads[payload_index];
                let marker = format!("§{}§", position.name);
                modified = modified.replace(&marker, payload);
                used_payloads.push(payload.clone());
                combination_index /= config.payloads.len();
            }

            requests.push((modified, used_payloads.join(", ")));
        }

        Ok(requests)
    }

    pub fn add_result(&self, result: IntruderResult) {
        self.results.write().push(result);
    }

    pub fn get_results(&self) -> Vec<IntruderResult> {
        self.results.read().clone()
    }

    pub fn clear_results(&self) {
        self.results.write().clear();
    }
}

fn parse_request(raw: &str) -> Result<Request<ProxyBody>> {
    let mut lines = raw.lines();
    let first_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty request"))?;
    let mut parts = first_line.split_whitespace();
    let method_str = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing method"))?;
    let uri_str = parts.next().ok_or_else(|| anyhow::anyhow!("Missing URI"))?;

    let method = Method::from_bytes(method_str.as_bytes())?;
    let uri = uri_str.parse::<Uri>()?;

    let mut builder = Request::builder().method(method).uri(uri);

    let mut body_start = false;
    let mut body_content = String::new();

    for line in lines {
        if body_start {
            body_content.push_str(line);
            body_content.push('\n');
        } else if line.is_empty() {
            body_start = true;
        } else if let Some((k, v)) = line.split_once(':') {
            builder = builder.header(k.trim(), v.trim());
        }
    }

    // Trim trailing newline from body if added
    if body_content.ends_with('\n') {
        body_content.pop();
    }

    Ok(builder.body(ProxyBody::from(bytes::Bytes::from(body_content)))?)
}

/// A small set of plain, commonly-used fuzzing inputs for payload positions.
///
/// These are generic probe values: empty input, numeric boundaries, booleans,
/// a few common account names, and an over-long string. They are deliberately
/// not exploit strings; this is a defensive testing tool, so building a real
/// attack wordlist is left to the operator.
pub fn common_payloads() -> Vec<String> {
    [
        "",
        "0",
        "1",
        "-1",
        "true",
        "false",
        "null",
        "admin",
        "test",
        "guest",
        "user",
        "2147483647",
        "-2147483648",
        "99999999999999999999",
        "          ",
        &"A".repeat(256),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

impl Default for Intruder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config(
        positions: Vec<&str>,
        payloads: Vec<&str>,
        attack_type: AttackType,
    ) -> IntruderConfig {
        IntruderConfig {
            positions: positions
                .into_iter()
                .enumerate()
                .map(|(i, name)| IntruderPosition {
                    start: i * 10,
                    end: i * 10 + 5,
                    name: name.to_string(),
                })
                .collect(),
            payloads: payloads.into_iter().map(String::from).collect(),
            attack_type,
            options: IntruderOptions::default(),
        }
    }

    #[test]
    fn test_sniper_attack() {
        let intruder = Intruder::new();
        let config = create_config(vec!["pos1", "pos2"], vec!["A", "B"], AttackType::Sniper);
        let template = "param1=§pos1§&param2=§pos2§";

        let results = intruder.generate_requests(template, &config).unwrap();
        let requests: Vec<String> = results.into_iter().map(|(r, _)| r).collect();

        // Sniper: each payload in each position = 2 payloads * 2 positions = 4
        assert_eq!(requests.len(), 4);

        // Check that each request has one position filled and others empty
        assert!(requests.contains(&"param1=A&param2=".to_string()));
        assert!(requests.contains(&"param1=&param2=A".to_string()));
        assert!(requests.contains(&"param1=B&param2=".to_string()));
        assert!(requests.contains(&"param1=&param2=B".to_string()));
    }

    #[test]
    fn test_battering_attack() {
        let intruder = Intruder::new();
        let config = create_config(vec!["pos1", "pos2"], vec!["X", "Y"], AttackType::Battering);
        let template = "a=§pos1§&b=§pos2§";

        let results = intruder.generate_requests(template, &config).unwrap();
        let requests: Vec<String> = results.into_iter().map(|(r, _)| r).collect();

        // Battering: same payload in all positions = 2 payloads
        assert_eq!(requests.len(), 2);
        assert!(requests.contains(&"a=X&b=X".to_string()));
        assert!(requests.contains(&"a=Y&b=Y".to_string()));
    }

    #[test]
    fn test_pitchfork_attack() {
        let intruder = Intruder::new();
        let config = create_config(
            vec!["user", "pass"],
            vec!["admin", "secret"],
            AttackType::Pitchfork,
        );
        let template = "username=§user§&password=§pass§";

        let results = intruder.generate_requests(template, &config).unwrap();
        let requests: Vec<String> = results.into_iter().map(|(r, _)| r).collect();

        // Pitchfork: parallel iteration = min(payloads, positions) iterations
        assert_eq!(requests.len(), 2);
        // Both positions get same index payload
        assert!(requests.contains(&"username=admin&password=admin".to_string()));
        assert!(requests.contains(&"username=secret&password=secret".to_string()));
    }

    #[test]
    fn test_cluster_bomb_attack() {
        let intruder = Intruder::new();
        let config = create_config(vec!["p1", "p2"], vec!["1", "2"], AttackType::ClusterBomb);
        let template = "x=§p1§&y=§p2§";

        let results = intruder.generate_requests(template, &config).unwrap();
        let requests: Vec<String> = results.into_iter().map(|(r, _)| r).collect();

        // Cluster bomb: all combinations = 2^2 = 4
        assert_eq!(requests.len(), 4);
        assert!(requests.contains(&"x=1&y=1".to_string()));
        assert!(requests.contains(&"x=1&y=2".to_string()));
        assert!(requests.contains(&"x=2&y=1".to_string()));
        assert!(requests.contains(&"x=2&y=2".to_string()));
    }

    #[test]
    fn test_cluster_bomb_empty_positions() {
        let intruder = Intruder::new();
        let config = IntruderConfig {
            positions: vec![],
            payloads: vec!["a".to_string()],
            attack_type: AttackType::ClusterBomb,
            options: IntruderOptions::default(),
        };

        let requests = intruder.generate_requests("template", &config).unwrap();
        assert!(requests.is_empty());
    }

    #[test]
    fn test_results_management() {
        let intruder = Intruder::new();

        // Initially empty
        assert!(intruder.get_results().is_empty());

        // Add results
        intruder.add_result(IntruderResult {
            request_id: 1,
            payload: "test1".to_string(),
            status_code: 200,
            response_length: 100,
            duration_ms: 50,
        });
        intruder.add_result(IntruderResult {
            request_id: 2,
            payload: "test2".to_string(),
            status_code: 404,
            response_length: 50,
            duration_ms: 30,
        });

        let results = intruder.get_results();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status_code, 200);
        assert_eq!(results[1].status_code, 404);

        // Clear results
        intruder.clear_results();
        assert!(intruder.get_results().is_empty());
    }

    #[test]
    fn test_single_position_sniper() {
        let intruder = Intruder::new();
        let config = create_config(vec!["id"], vec!["1", "2", "3"], AttackType::Sniper);
        let template = "/user/§id§";

        let results = intruder.generate_requests(template, &config).unwrap();
        let requests: Vec<String> = results.into_iter().map(|(r, _)| r).collect();

        assert_eq!(requests.len(), 3);
        assert!(requests.contains(&"/user/1".to_string()));
        assert!(requests.contains(&"/user/2".to_string()));
        assert!(requests.contains(&"/user/3".to_string()));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let intruder = Intruder::new();
        let intruder_clone = intruder.results.clone();

        let handle = thread::spawn(move || {
            intruder_clone.write().push(IntruderResult {
                request_id: 999,
                payload: "threaded".to_string(),
                status_code: 200,
                response_length: 10,
                duration_ms: 5,
            });
        });

        handle.join().unwrap();
        let results = intruder.get_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request_id, 999);
    }

    #[test]
    fn test_common_payloads_non_empty_and_plain() {
        let payloads = common_payloads();
        assert!(!payloads.is_empty());
        // Defensive tool: the built-in set must not ship exploit strings.
        for p in &payloads {
            assert!(!p.contains("DROP TABLE"));
            assert!(!p.contains("<script"));
            assert!(!p.contains("../"));
            assert!(!p.contains("OR '1'='1"));
        }
    }

    #[test]
    fn test_status_tracks_results() {
        let intruder = Intruder::new();
        let before = intruder.status();
        assert!(!before.running);
        assert_eq!(before.result_count, 0);

        intruder.add_result(IntruderResult {
            request_id: 1,
            payload: "p".to_string(),
            status_code: 200,
            response_length: 1,
            duration_ms: 1,
        });

        let after = intruder.status();
        assert!(!after.running);
        assert_eq!(after.result_count, 1);
    }

    /// A hung upstream must not leave the attack stuck after stop: wait until
    /// a probe is in flight, stop, then confirm running clears well before the
    /// request timeout (abort path), and that a fresh start is accepted.
    #[tokio::test]
    async fn stop_clears_running_when_upstream_hangs() {
        use crate::connection_pool::{install_crypto_provider, ConnectionPool};
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
        use tokio::io::AsyncReadExt;

        install_crypto_provider();

        let arrived = Arc::new(AtomicUsize::new(0));
        let upstream = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("upstream bind");
        let upstream_addr = upstream.local_addr().unwrap();
        let arrived_up = arrived.clone();
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = match upstream.accept().await {
                    Ok(pair) => pair,
                    Err(_) => return,
                };
                let arrived = arrived_up.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut tmp = [0u8; 1024];
                    loop {
                        let n = match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => n,
                        };
                        buf.extend_from_slice(&tmp[..n]);
                        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    // Probe is waiting on this hung response; stop must abort it.
                    arrived.fetch_add(1, AtomicOrdering::SeqCst);
                    std::future::pending::<()>().await;
                });
            }
        });

        let intruder = Intruder::new();
        let pool = ConnectionPool::new();
        let template = format!(
            "GET http://{upstream_addr}/probe-\u{00a7}p\u{00a7} HTTP/1.1\r\nHost: {upstream_addr}\r\nConnection: close\r\n\r\n"
        );
        const TIMEOUT_MS: u64 = 10_000;
        let config = IntruderConfig {
            positions: vec![IntruderPosition {
                start: 0,
                end: 0,
                name: "p".to_string(),
            }],
            payloads: vec!["one".to_string(), "two".to_string()],
            attack_type: AttackType::Sniper,
            options: IntruderOptions {
                concurrency: 2,
                delay_ms: 0,
                // Long enough that abort-on-stop is what clears running, not the timer.
                timeout_ms: TIMEOUT_MS,
            },
        };

        intruder
            .start_attack(template.clone(), config.clone(), pool.clone())
            .await
            .expect("first start");

        // Wait until upstream has accepted at least one in-flight probe.
        let mut saw_probe = false;
        for _ in 0..50 {
            if arrived.load(AtomicOrdering::SeqCst) > 0 {
                saw_probe = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }
        assert!(saw_probe, "upstream never received an in-flight probe");
        assert!(
            intruder.is_running(),
            "attack should still be running with a hung probe"
        );

        let stop_started = Instant::now();
        intruder.stop_attack();

        let mut drained = false;
        for _ in 0..50 {
            if !intruder.is_running() {
                drained = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }
        let elapsed_ms = stop_started.elapsed().as_millis() as u64;
        assert!(
            drained,
            "stop left the attack running while upstream was hung"
        );
        assert!(
            elapsed_ms < TIMEOUT_MS / 2,
            "running cleared after {elapsed_ms}ms; abort should beat the {TIMEOUT_MS}ms timeout"
        );

        intruder
            .start_attack(template, config, pool)
            .await
            .expect("start after stop must succeed");
        assert!(intruder.is_running());
        intruder.stop_attack();
    }

    /// Headers without a complete body should still record the HTTP status.
    #[tokio::test]
    async fn truncated_body_still_records_status() {
        use crate::connection_pool::{install_crypto_provider, ConnectionPool};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        install_crypto_provider();

        let upstream = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("upstream bind");
        let upstream_addr = upstream.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = upstream.accept().await.expect("accept");
            let mut buf = Vec::new();
            let mut tmp = [0u8; 1024];
            loop {
                let n = match sock.read(&mut tmp).await {
                    Ok(0) | Err(_) => return,
                    Ok(n) => n,
                };
                buf.extend_from_slice(&tmp[..n]);
                if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            // Promise a body, then close before sending it.
            let _ = sock
                .write_all(b"HTTP/1.1 203 Non-Authoritative Information\r\nContent-Length: 64\r\nConnection: close\r\n\r\n")
                .await;
            // Drop sock without writing the body.
        });

        let intruder = Intruder::new();
        let pool = ConnectionPool::new();
        let template = format!(
            "GET http://{upstream_addr}/probe-\u{00a7}p\u{00a7} HTTP/1.1\r\nHost: {upstream_addr}\r\nConnection: close\r\n\r\n"
        );
        let config = IntruderConfig {
            positions: vec![IntruderPosition {
                start: 0,
                end: 0,
                name: "p".to_string(),
            }],
            payloads: vec!["only".to_string()],
            attack_type: AttackType::Sniper,
            options: IntruderOptions {
                concurrency: 1,
                delay_ms: 0,
                timeout_ms: 5_000,
            },
        };

        intruder
            .start_attack(template, config, pool)
            .await
            .expect("start");

        let mut done = false;
        for _ in 0..50 {
            let status = intruder.status();
            if !status.running && status.result_count == 1 {
                done = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }
        assert!(done, "attack did not finish with one result");

        let results = intruder.get_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status_code, 203);
        assert_eq!(results[0].response_length, 0);
    }
}
