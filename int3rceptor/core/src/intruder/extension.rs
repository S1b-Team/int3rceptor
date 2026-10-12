//! Extension seam for Intruder payload sources and attack strategies.
//!
//! The public build always resolves to [`CommonPayloadSource`] and
//! [`PublicAttackStrategy`], which preserve today's built-in behavior.
//! Under the `pro` feature, [`pro_extension`] is the hook where
//! int3rceptor-pro can supply premium payload/strategy intelligence.
//! That private crate is not part of this repository; see
//! `docs/adr/0001-public-vs-pro-split.md`.

use super::{AttackType, IntruderConfig};
use anyhow::Result;

/// Supplies payloads for Intruder positions.
pub trait PayloadSource: Send + Sync {
    fn payloads(&self) -> Vec<String>;
}

/// Expands a request template and [`IntruderConfig`] into concrete requests.
///
/// Each entry is `(request_string, payload_label)`.
pub trait AttackStrategy: Send + Sync {
    fn generate(&self, template: &str, config: &IntruderConfig) -> Result<Vec<(String, String)>>;
}

/// Public built-in payload source: the same plain probe values as
/// [`super::common_payloads`].
#[derive(Debug, Default, Clone, Copy)]
pub struct CommonPayloadSource;

impl PayloadSource for CommonPayloadSource {
    fn payloads(&self) -> Vec<String> {
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
}

/// Public built-in attack strategy: Sniper, Battering, Pitchfork, ClusterBomb.
#[derive(Debug, Default, Clone, Copy)]
pub struct PublicAttackStrategy;

impl AttackStrategy for PublicAttackStrategy {
    fn generate(&self, template: &str, config: &IntruderConfig) -> Result<Vec<(String, String)>> {
        match config.attack_type {
            AttackType::Sniper => self.generate_sniper(template, config),
            AttackType::Battering => self.generate_battering(template, config),
            AttackType::Pitchfork => self.generate_pitchfork(template, config),
            AttackType::ClusterBomb => self.generate_cluster_bomb(template, config),
        }
    }
}

impl PublicAttackStrategy {
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
}

/// Resolve the active payload source for this build.
///
/// Without `pro`, this is always [`CommonPayloadSource`]. With `pro`, the
/// hook in [`pro_extension`] may return a premium source (wired from
/// int3rceptor-pro); the public tree falls back to the common set so the
/// feature still type-checks here.
pub fn resolve_payload_source() -> Box<dyn PayloadSource> {
    #[cfg(feature = "pro")]
    {
        return pro_extension::resolve_payload_source();
    }
    #[cfg(not(feature = "pro"))]
    {
        Box::new(CommonPayloadSource)
    }
}

/// Convenience: payloads from the build-selected [`PayloadSource`].
pub fn active_payloads() -> Vec<String> {
    resolve_payload_source().payloads()
}

/// Resolve the active attack strategy for this build.
///
/// Without `pro`, this is always [`PublicAttackStrategy`]. With `pro`, see
/// [`pro_extension`].
pub fn resolve_attack_strategy() -> Box<dyn AttackStrategy> {
    #[cfg(feature = "pro")]
    {
        return pro_extension::resolve_attack_strategy();
    }
    #[cfg(not(feature = "pro"))]
    {
        Box::new(PublicAttackStrategy)
    }
}

/// Pro-feature hooks for premium Intruder intelligence.
///
/// int3rceptor-pro owns the real premium payload sources and strategies.
/// In this public checkout the hooks keep the built-in implementations.
/// The full `--features pro` build still requires the private `core-pro`
/// files and dependency wiring used by the scanner. The pro workspace
/// replaces or overlays these hooks when building the commercial product.
#[cfg(feature = "pro")]
pub mod pro_extension {
    use super::{AttackStrategy, CommonPayloadSource, PayloadSource, PublicAttackStrategy};

    /// Hook: return the payload source used when the `pro` feature is on.
    pub fn resolve_payload_source() -> Box<dyn PayloadSource> {
        // Placeholder for int3rceptor-pro. Public fallback preserves behavior.
        Box::new(CommonPayloadSource)
    }

    /// Hook: return the attack strategy used when the `pro` feature is on.
    pub fn resolve_attack_strategy() -> Box<dyn AttackStrategy> {
        // Placeholder for int3rceptor-pro. Public fallback preserves behavior.
        Box::new(PublicAttackStrategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intruder::{AttackType, IntruderConfig, IntruderOptions, IntruderPosition};

    #[test]
    fn common_payload_source_matches_legacy_helper() {
        let expected: Vec<String> = [
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
        ]
        .iter()
        .map(|s| s.to_string())
        .chain(std::iter::once("A".repeat(256)))
        .collect();

        assert_eq!(CommonPayloadSource.payloads(), expected);
        assert_eq!(crate::intruder::common_payloads(), expected);
    }

    #[cfg(not(feature = "pro"))]
    #[test]
    fn resolve_defaults_to_public_impls() {
        let payloads = resolve_payload_source().payloads();
        assert_eq!(payloads, CommonPayloadSource.payloads());

        let config = IntruderConfig {
            positions: vec![IntruderPosition {
                start: 0,
                end: 0,
                name: "p".to_string(),
            }],
            payloads: vec!["A".to_string(), "B".to_string()],
            attack_type: AttackType::Sniper,
            options: IntruderOptions::default(),
        };
        let via_resolve = resolve_attack_strategy()
            .generate("x=§p§", &config)
            .unwrap();
        let via_public = PublicAttackStrategy.generate("x=§p§", &config).unwrap();
        assert_eq!(via_resolve, via_public);
        assert_eq!(via_resolve.len(), 2);
    }
}
