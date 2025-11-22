use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderConfig {
    pub positions: Vec<IntruderPosition>,
    pub payloads: Vec<String>,
    pub attack_type: AttackType,
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
}

impl Intruder {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(Vec::new())),
        }
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
        self.results.write().unwrap().push(result);
    }

    pub fn get_results(&self) -> Vec<IntruderResult> {
        self.results.read().unwrap().clone()
    }

    pub fn clear_results(&self) {
        self.results.write().unwrap().clear();
    }
}

impl Default for Intruder {
    fn default() -> Self {
        Self::new()
    }
}
