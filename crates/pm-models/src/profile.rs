//! Application Profile data model.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::protocol::RecordMeta;

/// Type of an application fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FingerprintType {
    Tag,
    Cmdline,
    Env,
    Path,
    #[default]
    Unknown,
}


/// Operation of a fingerprint match.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FingerprintOperation {
    #[default]
    Equal,
    Prefix,
    Regex,
    Unknown,
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
    #[serde(alias = "ConnectionsCount", alias = "countAliveConnections", default)]
    pub connections_count: u64,
    #[serde(alias = "BlockedCount", alias = "countBlocked", default)]
    pub blocked_count: u64,
    #[serde(alias = "BytesSent", default)]
    pub bytes_sent: u64,
    #[serde(alias = "BytesReceived", default)]
    pub bytes_received: u64,
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

    fn read_bool_option(&self, flat_key: &str, section: &str, nested_key: &str) -> Option<bool> {
        let cfg = self.config.as_ref()?;
        if let Some(enabled) = cfg.get(flat_key).and_then(Value::as_bool) {
            return Some(enabled);
        }
        cfg.get(section)
            .and_then(|obj| obj.get(nested_key))
            .and_then(Value::as_bool)
    }

    fn write_bool_option(
        &mut self,
        flat_key: &str,
        section: &str,
        nested_key: &str,
        value: Option<bool>,
    ) {
        let mut map = match self.config.take() {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        match value {
            Some(enabled) => {
                map.insert(flat_key.to_string(), Value::Bool(enabled));
                if let Some(obj) = map.get_mut(section).and_then(Value::as_object_mut) {
                    obj.remove(nested_key);
                }
            }
            None => {
                map.remove(flat_key);
                let remove_section = if let Some(obj) = map.get_mut(section).and_then(Value::as_object_mut) {
                    obj.remove(nested_key);
                    obj.is_empty()
                } else {
                    false
                };
                if remove_section {
                    map.remove(section);
                }
            }
        }
        self.config = if map.is_empty() {
            None
        } else {
            Some(Value::Object(map))
        };
    }

    /// Returns the per-profile VPN override, or `None` when the global
    /// `spn/use` default applies.
    pub fn vpn_route_override(&self) -> Option<bool> {
        self.read_bool_option("spn/use", "spn", "use")
    }

    /// Returns true if this profile is configured to route via VPN (Xray),
    /// ignoring the global default.
    pub fn route_via_vpn(&self) -> bool {
        self.vpn_route_override().unwrap_or(false)
    }

    /// Returns the effective VPN state with the global `spn/use` default.
    pub fn route_via_vpn_with_default(&self, global_default: bool) -> bool {
        self.vpn_route_override().unwrap_or(global_default)
    }

    /// Sets or clears the per-app VPN override (`None` inherits global `spn/use`).
    pub fn set_vpn_route_override(&mut self, override_val: Option<bool>) {
        self.write_bool_option("spn/use", "spn", "use", override_val);
    }

    /// Updates this profile's configuration to enable or disable VPN routing.
    pub fn set_route_via_vpn(&mut self, route_vpn: bool) {
        self.set_vpn_route_override(Some(route_vpn));
    }

    /// Returns explicit Split Tunnel override if set (`splittun/use`), or `None` when inheriting global.
    pub fn split_tun_override(&self) -> Option<bool> {
        self.read_bool_option("splittun/use", "splittun", "use")
    }

    /// Returns the effective Split Tunnel state with the global `splittun/use` default.
    pub fn route_via_split_tun_with_default(&self, global_default: bool) -> bool {
        self.split_tun_override().unwrap_or(global_default)
    }

    /// Sets or clears the per-app Split Tunnel override (`None` inherits global `splittun/use`).
    pub fn set_split_tun_override(&mut self, override_val: Option<bool>) {
        self.write_bool_option("splittun/use", "splittun", "use", override_val);
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
