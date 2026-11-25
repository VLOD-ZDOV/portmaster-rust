//! Network connection data models for Portmaster Netquery.

use serde::{Deserialize, Deserializer, Serialize};

fn map_verdict(v: i64) -> String {
    match v {
        2 | 5 | 6 => "permit".to_string(),
        3 => "block".to_string(),
        4 => "drop".to_string(),
        7 => "ask".to_string(),
        _ => "permit".to_string(),
    }
}

fn deserialize_verdict<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct VerdictVisitor;
    impl<'de> serde::de::Visitor<'de> for VerdictVisitor {
        type Value = String;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or integer representing verdict")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(map_verdict(v))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(map_verdict(v as i64))
        }
    }
    deserializer.deserialize_any(VerdictVisitor)
}

fn map_proto(v: i64) -> String {
    match v {
        6 => "TCP".to_string(),
        17 => "UDP".to_string(),
        1 | 58 => "ICMP".to_string(),
        other => format!("IP-{other}"),
    }
}

fn deserialize_protocol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct ProtoVisitor;
    impl<'de> serde::de::Visitor<'de> for ProtoVisitor {
        type Value = String;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or integer representing protocol")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(map_proto(v))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(map_proto(v as i64))
        }
    }
    deserializer.deserialize_any(ProtoVisitor)
}

fn map_scope(v: i64) -> String {
    match v {
        0 => "Undefined".to_string(),
        1 => "HostLocal".to_string(),
        2 => "LinkLocal".to_string(),
        3 => "LAN".to_string(),
        4 => "Internet".to_string(),
        5 => "LocalMulticast".to_string(),
        6 => "GlobalMulticast".to_string(),
        _ => "Internet".to_string(),
    }
}

fn deserialize_scope<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct ScopeVisitor;
    impl<'de> serde::de::Visitor<'de> for ScopeVisitor {
        type Value = String;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or integer representing scope")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(map_scope(v))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(map_scope(v as i64))
        }
    }
    deserializer.deserialize_any(ScopeVisitor)
}

fn deserialize_bool_flexible<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoolVisitor;
    impl<'de> serde::de::Visitor<'de> for BoolVisitor {
        type Value = bool;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or integer 0/1")
        }
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v != 0)
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v != 0)
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.eq_ignore_ascii_case("true") || v == "1")
        }
    }
    deserializer.deserialize_any(BoolVisitor)
}

/// Represents a single network connection event from Portmaster Netquery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectionRecord {
    #[serde(alias = "ID", default)]
    pub id: String,
    #[serde(alias = "Domain", default)]
    pub domain: Option<String>,
    #[serde(alias = "RemoteIP", default)]
    pub remote_ip: String,
    #[serde(alias = "RemotePort", default)]
    pub remote_port: u16,
    #[serde(alias = "Protocol", alias = "IPProtocol", alias = "ip_protocol", default, deserialize_with = "deserialize_protocol")]
    pub protocol: String,
    #[serde(alias = "Direction", default)]
    pub direction: String,
    #[serde(alias = "Scope", default, deserialize_with = "deserialize_scope")]
    pub scope: String,
    #[serde(alias = "Country", default)]
    pub country: Option<String>,
    #[serde(alias = "CountryName", default)]
    pub country_name: Option<String>,
    #[serde(alias = "ProfileID", alias = "profile", default)]
    pub profile_id: String,
    #[serde(alias = "ProfileName", default)]
    pub profile_name: String,
    #[serde(alias = "Verdict", default, deserialize_with = "deserialize_verdict")]
    pub verdict: String, // "permit", "block", "drop", "ask"
    #[serde(alias = "VerdictReason", default)]
    pub verdict_reason: String,
    #[serde(alias = "ASOwner", default)]
    pub as_owner: Option<String>,
    #[serde(alias = "BytesSent", default)]
    pub bytes_sent: u64,
    #[serde(alias = "BytesReceived", default)]
    pub bytes_received: u64,
    #[serde(alias = "TimeAgo", default)]
    pub time_ago: String,
    #[serde(alias = "Started", default)]
    pub started: Option<String>,
    #[serde(alias = "Active", default, deserialize_with = "deserialize_bool_flexible")]
    pub active: bool,
}

impl ConnectionRecord {
    pub fn is_permitted(&self) -> bool {
        self.verdict.eq_ignore_ascii_case("permit") || self.verdict.eq_ignore_ascii_case("allow")
    }

    pub fn is_blocked(&self) -> bool {
        self.verdict.eq_ignore_ascii_case("block") || self.verdict.eq_ignore_ascii_case("drop")
    }

    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_netquery_row() {
        let json = r#"{
            "id": "conn-123",
            "profile": "local/xyz",
            "profile_name": "Firefox",
            "domain": "example.com",
            "remote_ip": "93.184.216.34",
            "remote_port": 443,
            "ip_protocol": 6,
            "direction": "outbound",
            "scope": 4,
            "country": "US",
            "verdict": 2,
            "bytes_sent": 1200,
            "bytes_received": 34000,
            "active": true
        }"#;

        let record: ConnectionRecord = serde_json::from_str(json).expect("deserialize connection record");
        assert_eq!(record.id, "conn-123");
        assert_eq!(record.profile_id, "local/xyz");
        assert_eq!(record.profile_name, "Firefox");
        assert_eq!(record.protocol, "TCP");
        assert_eq!(record.scope, "Internet");
        assert_eq!(record.verdict, "permit");
        assert!(record.is_permitted());
        assert_eq!(record.bytes_sent, 1200);
        assert_eq!(record.bytes_received, 34000);
        assert!(record.active);
        assert_eq!(ConnectionRecord::format_bytes(1200), "1.2 KB");
        assert_eq!(ConnectionRecord::format_bytes(34000), "33.2 KB");
    }
}
