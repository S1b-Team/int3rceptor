use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

/// IP filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpFilterConfig {
    /// Allowed IP addresses (allowlist)
    pub allowed_ips: Vec<String>,
    /// Blocked IP addresses (blocklist)
    pub blocked_ips: Vec<String>,
    /// Allow private IP ranges (127.0.0.0/8, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
    #[serde(default = "default_allow_private")]
    pub allow_private: bool,
    /// Allow loopback addresses
    #[serde(default = "default_allow_loopback")]
    pub allow_loopback: bool,
    /// Operating mode: "allowlist", "blocklist", or "off"
    #[serde(default = "default_mode")]
    pub mode: String,
}

impl Default for IpFilterConfig {
    fn default() -> Self {
        Self {
            allowed_ips: Vec::new(),
            blocked_ips: Vec::new(),
            allow_private: true,
            allow_loopback: true,
            mode: "off".to_string(),
        }
    }
}

fn default_allow_private() -> bool {
    true
}

fn default_allow_loopback() -> bool {
    true
}

fn default_mode() -> String {
    "off".to_string()
}

/// IP filter manager
pub struct IpFilter {
    config: RwLock<IpFilterConfig>,
    allowed_ips: RwLock<HashSet<IpAddr>>,
    blocked_ips: RwLock<HashSet<IpAddr>>,
}

impl IpFilter {
    /// Create a new IP filter with the given configuration
    pub fn new(config: IpFilterConfig) -> Self {
        let allowed_ips = config
            .allowed_ips
            .iter()
            .filter_map(|ip| ip.parse::<IpAddr>().ok())
            .collect();

        let blocked_ips = config
            .blocked_ips
            .iter()
            .filter_map(|ip| ip.parse::<IpAddr>().ok())
            .collect();

        Self {
            config: RwLock::new(config),
            allowed_ips: RwLock::new(allowed_ips),
            blocked_ips: RwLock::new(blocked_ips),
        }
    }

    /// Check if an IP address is allowed
    pub fn is_allowed(&self, ip: IpAddr) -> bool {
        let config = match self.config.read() {
            Ok(c) => c,
            Err(_) => return false,
        };

        // If filtering is off, allow all
        if config.mode == "off" {
            return true;
        }

        // Check loopback
        if ip.is_loopback() {
            return config.allow_loopback;
        }

        // Check private IPs
        if Self::is_private_ip(ip) && !config.allow_private {
            return false;
        }

        // Check blocklist first (blocklist takes precedence)
        let blocked = match self.blocked_ips.read() {
            Ok(ips) => ips.contains(&ip),
            Err(_) => false,
        };

        if blocked {
            return false;
        }

        // In allowlist mode, check if IP is explicitly allowed
        if config.mode == "allowlist" {
            let allowed = match self.allowed_ips.read() {
                Ok(ips) => ips.contains(&ip),
                Err(_) => false,
            };

            allowed || (config.allow_private && Self::is_private_ip(ip))
        } else {
            // In blocklist or off mode, allow by default if not blocked
            true
        }
    }

    /// Check if an IP is in a private range
    fn is_private_ip(ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 10.0.0.0/8
                octets[0] == 10
                    // 172.16.0.0/12
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    // 192.168.0.0/16
                    || (octets[0] == 192 && octets[1] == 168)
                    // 169.254.0.0/16 (link-local)
                    || (octets[0] == 169 && octets[1] == 254)
            }
            IpAddr::V6(ipv6) => {
                // fc00::/7 (Unique Local Addresses)
                (ipv6.segments()[0] & 0xfe00) == 0xfc00
                    // fe80::/10 (Link-Local)
                    || (ipv6.segments()[0] & 0xffc0) == 0xfe80
            }
        }
    }

    /// Add an IP to the allowlist
    pub fn add_allowed_ip(&self, ip: IpAddr) {
        if let Ok(mut ips) = self.allowed_ips.write() {
            ips.insert(ip);
        }
        self.update_config();
    }

    /// Add an IP to the blocklist
    pub fn add_blocked_ip(&self, ip: IpAddr) {
        if let Ok(mut ips) = self.blocked_ips.write() {
            ips.insert(ip);
        }
        self.update_config();
    }

    /// Remove an IP from the allowlist
    pub fn remove_allowed_ip(&self, ip: &IpAddr) {
        if let Ok(mut ips) = self.allowed_ips.write() {
            ips.remove(ip);
        }
        self.update_config();
    }

    /// Remove an IP from the blocklist
    pub fn remove_blocked_ip(&self, ip: &IpAddr) {
        if let Ok(mut ips) = self.blocked_ips.write() {
            ips.remove(ip);
        }
        self.update_config();
    }

    /// Update configuration
    pub fn set_config(&self, new_config: IpFilterConfig) {
        let allowed_ips: HashSet<IpAddr> = new_config
            .allowed_ips
            .iter()
            .filter_map(|ip| ip.parse().ok())
            .collect();

        let blocked_ips: HashSet<IpAddr> = new_config
            .blocked_ips
            .iter()
            .filter_map(|ip| ip.parse().ok())
            .collect();

        if let Ok(mut ips) = self.allowed_ips.write() {
            *ips = allowed_ips;
        }

        if let Ok(mut ips) = self.blocked_ips.write() {
            *ips = blocked_ips;
        }

        if let Ok(mut config) = self.config.write() {
            *config = new_config;
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> IpFilterConfig {
        self.config.read().unwrap().clone()
    }

    /// Update the config with current IP sets
    fn update_config(&self) {
        if let Ok(mut config) = self.config.write() {
            if let Ok(allowed) = self.allowed_ips.read() {
                config.allowed_ips = allowed.iter().map(|ip| ip.to_string()).collect();
            }
            if let Ok(blocked) = self.blocked_ips.read() {
                config.blocked_ips = blocked.iter().map(|ip| ip.to_string()).collect();
            }
        }
    }
}

impl Default for IpFilter {
    fn default() -> Self {
        Self::new(IpFilterConfig::default())
    }
}

/// Axum middleware for IP filtering
pub async fn ip_filter_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(filter): State<Arc<IpFilter>>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    if !filter.is_allowed(ip) {
        tracing::warn!(
            ip = %ip,
            path = %req.uri(),
            "IP address blocked by filter"
        );

        return (
            StatusCode::FORBIDDEN,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Access denied: IP address not allowed"}"#,
        )
            .into_response();
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_filter_off_mode_allows_all() {
        let config = IpFilterConfig {
            mode: "off".to_string(),
            ..Default::default()
        };
        let filter = IpFilter::new(config);

        assert!(filter.is_allowed("1.2.3.4".parse().unwrap()));
        assert!(filter.is_allowed("192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn test_ip_filter_blocklist() {
        let config = IpFilterConfig {
            mode: "blocklist".to_string(),
            blocked_ips: vec!["1.2.3.4".to_string()],
            ..Default::default()
        };
        let filter = IpFilter::new(config);

        assert!(!filter.is_allowed("1.2.3.4".parse().unwrap()));
        assert!(filter.is_allowed("5.6.7.8".parse().unwrap()));
    }

    #[test]
    fn test_ip_filter_allowlist() {
        let config = IpFilterConfig {
            mode: "allowlist".to_string(),
            allowed_ips: vec!["1.2.3.4".to_string()],
            allow_private: false,
            allow_loopback: true,
            ..Default::default()
        };
        let filter = IpFilter::new(config);

        assert!(filter.is_allowed("1.2.3.4".parse().unwrap()));
        assert!(!filter.is_allowed("5.6.7.8".parse().unwrap()));
        assert!(filter.is_allowed("127.0.0.1".parse().unwrap())); // loopback allowed
    }

    #[test]
    fn test_private_ip_detection() {
        assert!(IpFilter::is_private_ip("10.0.0.1".parse().unwrap()));
        assert!(IpFilter::is_private_ip("172.16.0.1".parse().unwrap()));
        assert!(IpFilter::is_private_ip("192.168.1.1".parse().unwrap()));
        assert!(!IpFilter::is_private_ip("8.8.8.8".parse().unwrap()));
    }
}
