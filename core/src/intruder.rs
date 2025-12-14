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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderOptions {
    pub concurrency: usize,
    pub delay_ms: u64,
}

impl Default for IntruderOptions {
    fn default() -> Self {
        Self {
            concurrency: 1,
            delay_ms: 0,
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

pub struct Intruder {
    results: Arc<RwLock<Vec<IntruderResult>>>,
    is_running: Arc<AtomicBool>,
}

impl Intruder {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
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

        self.clear_results();
        let requests = self.generate_requests(&template, &config)?;
        let results = self.results.clone();
        let is_running = self.is_running.clone();
        let concurrency = config.options.concurrency.max(1);
        let delay = config.options.delay_ms;

        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
            let mut handles = Vec::new();

            for (id, req_str) in requests.into_iter().enumerate() {
                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let pool = pool.clone();
                let results = results.clone();
                let req_str = req_str.clone();

                if delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }

                handles.push(tokio::spawn(async move {
                    let _permit = permit;
                    let start = Instant::now();

                    // Parse and send
                    match parse_request(&req_str) {
                        Ok(req) => {
                            let client = pool.client();
                            match client.request(req).await {
                                Ok(resp) => {
                                    let status = resp.status().as_u16();
                                    let mut body_len = 0;
                                    // Read body to get length (and consume it)
                                    if let Ok(bytes) = resp.collect().await {
                                        body_len = bytes.to_bytes().len();
                                    }

                                    results.write().push(IntruderResult {
                                        request_id: id,
                                        payload: "".to_string(), // TODO: Extract payload from req_str or config
                                        status_code: status,
                                        response_length: body_len,
                                        duration_ms: start.elapsed().as_millis() as u64,
                                    });
                                }
                                Err(_) => {
                                    // Log error or store failed result
                                }
                            }
                        }
                        Err(_) => {
                            // Parse error
                        }
                    }
                }));
            }

            // Wait for all to finish
            for handle in handles {
                let _ = handle.await;
            }

            is_running.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    pub fn stop_attack(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn generate_requests(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<String>> {
        match config.attack_type {
            AttackType::Sniper => self.generate_sniper(template, config),
            AttackType::Battering => self.generate_battering(template, config),
            AttackType::Pitchfork => self.generate_pitchfork(template, config),
            AttackType::ClusterBomb => self.generate_cluster_bomb(template, config),
        }
    }

    fn generate_sniper(&self, template: &str, config: &IntruderConfig) -> Result<Vec<String>> {
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

                requests.push(modified);
            }
        }

        Ok(requests)
    }

    fn generate_battering(&self, template: &str, config: &IntruderConfig) -> Result<Vec<String>> {
        let mut requests = Vec::new();

        for payload in &config.payloads {
            let mut modified = template.to_string();

            for position in &config.positions {
                let marker = format!("§{}§", position.name);
                modified = modified.replace(&marker, payload);
            }

            requests.push(modified);
        }

        Ok(requests)
    }

    fn generate_pitchfork(&self, template: &str, config: &IntruderConfig) -> Result<Vec<String>> {
        let mut requests = Vec::new();
        let payload_count = config.payloads.len();
        let empty_string = String::new();

        for i in 0..payload_count {
            let mut modified = template.to_string();

            for position in config.positions.iter() {
                let marker = format!("§{}§", position.name);
                let payload = config.payloads.get(i).unwrap_or(&empty_string);
                modified = modified.replace(&marker, payload);
            }

            requests.push(modified);
        }

        Ok(requests)
    }

    fn generate_cluster_bomb(
        &self,
        template: &str,
        config: &IntruderConfig,
    ) -> Result<Vec<String>> {
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

            for position in &config.positions {
                let payload_index = combination_index % config.payloads.len();
                let payload = &config.payloads[payload_index];
                let marker = format!("§{}§", position.name);
                modified = modified.replace(&marker, payload);
                combination_index /= config.payloads.len();
            }

            requests.push(modified);
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

        let requests = intruder.generate_requests(template, &config).unwrap();

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

        let requests = intruder.generate_requests(template, &config).unwrap();

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

        let requests = intruder.generate_requests(template, &config).unwrap();

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

        let requests = intruder.generate_requests(template, &config).unwrap();

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

        let requests = intruder.generate_requests(template, &config).unwrap();

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
}
