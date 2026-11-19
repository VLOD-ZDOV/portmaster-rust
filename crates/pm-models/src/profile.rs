//! Application Profile data model.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::protocol::RecordMeta;

/// Type of an application fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FingerprintType {
    Tag,
    Cmdline,
    Env,
    Path,
    Unknown,
}

impl Default for FingerprintType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Operation of a fingerprint match.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FingerprintOperation {
    Equal,
    Prefix,
    Regex,
    Unknown,
}

impl Default for FingerprintOperation {
    fn default() -> Self {
        Self::Equal
    }
}

/// An application fingerprint descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProfileFingerprint {
    #[serde(rename = "Type", default)]
    pub fp_type: FingerprintType,
    #[serde(rename = "Key", default)]
    pub key: String,
    #[serde(rename = "Operation", default)]
    pub operation: FingerprintOperation,
    #[serde(rename = "Value", default)]
    pub value: String,
}

/// Icon reference for a profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProfileIcon {
    #[serde(rename = "Type", default)]
    pub icon_type: String, // "database", "path", "api", ""
    #[serde(rename = "Source", default)]
    pub source: String, // "user", "import", "core", "ui", ""
    #[serde(rename = "Value", default)]
    pub value: String,
}

fn default_local_source() -> String {
    "local".to_string()
}

/// Network activity stats for an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AppStats {
    #[serde(default)]
    pub connections_count: u64,
    #[serde(default)]
    pub blocked_count: u64,
    #[serde(default)]
    pub bytes_sent: u64,
    #[serde(default)]
    pub bytes_received: u64,
}

impl AppStats {
    pub fn blocked_percent(&self) -> f64 {
        if self.connections_count == 0 {
            0.0
        } else {
            (self.blocked_count as f64 / self.connections_count as f64) * 100.0
        }
    }
}

/// An application profile defining network security & routing settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AppProfile {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Source", default = "default_local_source")]
    pub source: String,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "Warning", default)]
    pub warning: String,
    #[serde(rename = "WarningLastUpdated", default)]
    pub warning_last_updated: Value,
    #[serde(rename = "Homepage", default)]
    pub homepage: String,
    #[serde(rename = "Icon", default)]
    pub icon: String,
    #[serde(rename = "IconType", default)]
    pub icon_type: String,
    #[serde(rename = "Icons", default)]
    pub icons: Vec<ProfileIcon>,
    #[serde(rename = "LinkedPath", default)]
    pub linked_path: String,
    #[serde(rename = "PresentationPath", default)]
    pub presentation_path: String,
    #[serde(rename = "Fingerprints", default)]
    pub fingerprints: Vec<ProfileFingerprint>,
    #[serde(rename = "Config", default)]
    pub config: Option<Value>,
    #[serde(rename = "Stats", default)]
    pub stats: Option<AppStats>,
    #[serde(rename = "Internal", default)]
    pub internal: bool,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RecordMeta>,
}

impl AppProfile {
    /// Constructs a basic application profile.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source: "local".to_string(),
            ..Default::default()
        }
    }

    /// Primary database key: `core:profiles/<source>/<id>`.
    pub fn database_key(&self) -> String {
        format!("core:profiles/{}/{}", self.source, self.id)
    }

    /// Returns the number of currently active connections.
    pub fn count_alive_connections(&self) -> u64 {
        self.stats.map(|s| s.connections_count).unwrap_or(0)
    }

    /// Best human-readable display name.
    pub fn display_name(&self) -> &str {
        if !self.name.trim().is_empty() {
            &self.name
        } else if !self.presentation_path.trim().is_empty() {
            self.presentation_path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(&self.presentation_path)
        } else {
            &self.id
        }
    }

    /// Returns true if this profile is configured to route via VPN/proxy.
    pub fn route_via_vpn(&self) -> bool {
        let Some(cfg) = &self.config else {
            return false;
        };

        // Check flat key "splittun/usagePolicy"
        if let Some(val) = cfg.get("splittun/usagePolicy") {
            if let Some(s) = val.as_str() {
                if s == "vpn" || s == "proxy" || s == "force-vpn" {
                    return true;
                }
            }
        }

        // Check nested structure: splittun -> usagePolicy
        if let Some(splittun) = cfg.get("splittun") {
            if let Some(val) = splittun.get("usagePolicy") {
                if let Some(s) = val.as_str() {
                    if s == "vpn" || s == "proxy" || s == "force-vpn" {
                        return true;
                    }
                }
            }
        }

        // Also check if proxy_url is configured
        if let Some(proxy_url) = cfg.get("proxy_url").or_else(|| cfg.get("proxy/url")) {
            if let Some(s) = proxy_url.as_str() {
                if !s.trim().is_empty() {
                    return true;
                }
            }
        }

        false
    }

    /// Updates this profile's configuration to enable or disable VPN routing.
    pub fn set_route_via_vpn(&mut self, route_vpn: bool) {
        let value_str = if route_vpn { "vpn" } else { "direct" };
        let mut map = match self.config.take() {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };

        map.insert(
            "splittun/usagePolicy".to_string(),
            Value::String(value_str.to_string()),
        );
        self.config = Some(Value::Object(map));
    }

    /// Returns default action ("permit", "block", "ask" or "default").
    pub fn default_action(&self) -> &str {
        let Some(cfg) = &self.config else {
            return "default";
        };

        if let Some(val) = cfg.get("filter/defaultAction") {
            if let Some(s) = val.as_str() {
                return s;
            }
        }

        if let Some(filter) = cfg.get("filter") {
            if let Some(val) = filter.get("defaultAction") {
                if let Some(s) = val.as_str() {
                    return s;
                }
            }
        }

        "default"
    }

    /// Sets default action.
    pub fn set_default_action(&mut self, action: &str) {
        let mut map = match self.config.take() {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };

        map.insert(
            "filter/defaultAction".to_string(),
            Value::String(action.to_string()),
        );
        self.config = Some(Value::Object(map));
    }
}
