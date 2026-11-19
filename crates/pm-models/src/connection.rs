//! Network connection data models for Portmaster Netquery.

use serde::{Deserialize, Serialize};

/// Represents a single network connection event from Portmaster Netquery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionRecord {
    pub id: String,
    #[serde(default)]
    pub domain: Option<String>,
    pub remote_ip: String,
    pub remote_port: u16,
    pub protocol: String,
    pub direction: String,
    pub scope: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub country_name: Option<String>,
    pub profile_id: String,
    pub profile_name: String,
    pub verdict: String, // "permit", "block", "drop", "ask"
    pub verdict_reason: String,
    #[serde(default)]
    pub as_owner: Option<String>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub time_ago: String,
    pub active: bool,
}

impl ConnectionRecord {
    pub fn is_permitted(&self) -> bool {
        self.verdict.eq_ignore_ascii_case("permit") || self.verdict.eq_ignore_ascii_case("allow")
    }

    pub fn is_blocked(&self) -> bool {
        self.verdict.eq_ignore_ascii_case("block") || self.verdict.eq_ignore_ascii_case("drop")
    }
}
