//! Xray VPN service data models and commands.

use serde::{Deserialize, Serialize};

/// Lifecycle state of the Xray background process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProcessState {
    #[default]
    Stopped,
    Starting,
    Running,
    Failed,
}

impl ProcessState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Failed => "failed",
        }
    }
}

/// Real-time traffic counters in bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrafficStatus {
    /// Bytes uploaded through the proxy outbound.
    #[serde(default)]
    pub uplink: u64,
    /// Bytes downloaded through the proxy outbound.
    #[serde(default)]
    pub downlink: u64,
}

impl TrafficStatus {
    /// Format bytes into human-readable string (KB, MB, GB).
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.0} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn formatted_uplink(&self) -> String {
        Self::format_bytes(self.uplink)
    }

    pub fn formatted_downlink(&self) -> String {
        Self::format_bytes(self.downlink)
    }
}

/// Overall status of the Xray VPN integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct XrayStatus {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub binary_available: bool,
    #[serde(default)]
    pub state: ProcessState,
    #[serde(rename = "nodeId", alias = "node_id", default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(rename = "nodeName", alias = "node_name", default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(rename = "inboundAddress", alias = "inbound_address", default, skip_serializing_if = "Option::is_none")]
    pub inbound_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub connected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traffic: Option<TrafficStatus>,
    #[serde(rename = "exitIp", alias = "exit_ip", default, skip_serializing_if = "Option::is_none")]
    pub exit_ip: Option<String>,
    #[serde(rename = "exitCountry", alias = "exit_country", default, skip_serializing_if = "Option::is_none")]
    pub exit_country: Option<String>,
}

/// One VPN node / endpoint extracted from a subscription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XrayNode {
    pub id: String,
    #[serde(rename = "subscriptionId", alias = "subscription_id", default)]
    pub subscription_id: String,
    pub name: String,
    #[serde(rename = "country", alias = "country_code", default)]
    pub country_code: String,
    #[serde(rename = "countryName", alias = "country_name", default)]
    pub country_name: String,
    #[serde(default)]
    pub protocol: String, // "vless", "vmess", "trojan", "shadowsocks", "hysteria", etc.
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub port: u16,
    #[serde(rename = "pingMs", alias = "ping_ms", default, skip_serializing_if = "Option::is_none")]
    pub ping_ms: Option<u32>,
}

impl XrayNode {
    /// Returns 2-letter country code in uppercase or "??"
    pub fn display_country_flag(&self) -> &str {
        if self.country_code.trim().is_empty() {
            "??"
        } else {
            &self.country_code
        }
    }
}

/// A subscription providing VPN nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XraySubscription {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "added_at", alias = "last_updated", default)]
    pub last_updated: i64,
    #[serde(default)]
    pub node_count: usize,
}

/// Request payload to connect to a specific Xray VPN node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectRequest {
    #[serde(rename = "nodeId", alias = "node_id")]
    pub node_id: String,
    #[serde(rename = "inboundPort", alias = "inbound_port", default, skip_serializing_if = "Option::is_none")]
    pub inbound_port: Option<u16>,
}
