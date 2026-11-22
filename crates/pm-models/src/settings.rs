//! Configuration setting and quick-setting data models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::RecordMeta;

/// Type of a configuration option value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OptionType {
    #[serde(rename = "any")]
    Any = 0,
    #[serde(rename = "string")]
    String = 1,
    #[serde(rename = "[]string")]
    StringArray = 2,
    #[serde(rename = "int")]
    Int = 3,
    #[serde(rename = "bool")]
    Bool = 4,
}

impl Default for OptionType {
    fn default() -> Self {
        Self::String
    }
}

/// Required user expertise level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ExpertiseLevel {
    #[default]
    User = 0,
    Expert = 1,
    Developer = 2,
}

/// Maturity and release level of an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ReleaseLevel {
    #[default]
    Stable = 0,
    Beta = 1,
    Experimental = 2,
}

fn default_quick_setting_action() -> String {
    "replace".to_string()
}

/// A pre-packaged quick setting choice attached to an option annotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuickSetting {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: Value,
    #[serde(rename = "Action", default = "default_quick_setting_action")]
    pub action: String, // "replace", "merge-top", "merge-bottom"
    #[serde(rename = "FlagID", skip_serializing_if = "Option::is_none")]
    pub flag_id: Option<String>,
}

/// An enumerated choice for an option.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PossibleValue {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "Value")]
    pub value: Value,
}

/// Well-known annotations on configuration options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SettingAnnotations {
    #[serde(rename = "safing/portbase:ui:display-hint", skip_serializing_if = "Option::is_none")]
    pub display_hint: Option<String>,
    #[serde(rename = "safing/portbase:ui:order", skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    #[serde(rename = "safing/portbase:ui:unit", skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "safing/portbase:ui:category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(rename = "safing/portbase:ui:quick-setting", skip_serializing_if = "Option::is_none")]
    pub quick_settings: Option<Vec<QuickSetting>>,
    #[serde(rename = "safing/portbase:options:restart-pending", skip_serializing_if = "Option::is_none")]
    pub restart_pending: Option<bool>,
}

/// A full Portmaster configuration setting definition and value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "Help", default)]
    pub help: String,
    #[serde(rename = "OptType")]
    pub opt_type: OptionType,
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(rename = "DefaultValue")]
    pub default_value: Value,
    #[serde(rename = "ExpertiseLevel", default)]
    pub expertise_level: u8,
    #[serde(rename = "ReleaseLevel", default)]
    pub release_level: u8,
    #[serde(rename = "Annotations", default)]
    pub annotations: SettingAnnotations,
    #[serde(rename = "PossibleValues", default)]
    pub possible_values: Vec<PossibleValue>,
    #[serde(rename = "RequiresRestart", default)]
    pub requires_restart: bool,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RecordMeta>,
}

impl Setting {
    /// Returns the active value if set, falling back to default_value.
    pub fn effective_value(&self) -> &Value {
        self.value.as_ref().unwrap_or(&self.default_value)
    }

    /// Returns true if custom value differs from default_value.
    pub fn is_modified(&self) -> bool {
        match &self.value {
            Some(v) => v != &self.default_value,
            None => false,
        }
    }

    /// Alias for is_modified matching Portmaster userConfigured flag.
    pub fn is_user_configured(&self) -> bool {
        self.is_modified()
    }

    /// Returns unit string if defined.
    pub fn unit(&self) -> Option<&str> {
        self.annotations.unit.as_deref()
    }

    /// Resets the setting to its default value by clearing the custom value.
    pub fn reset_to_default(&mut self) {
        self.value = None;
    }

    /// Returns the subsystem key prefix (e.g. "filter", "dns", "core", "xray", "splittun").
    pub fn subsystem(&self) -> &str {
        if let Some(pos) = self.key.find('/') {
            &self.key[..pos]
        } else {
            "core"
        }
    }

    /// Returns the category from annotations or defaults to "General".
    pub fn category(&self) -> &str {
        self.annotations
            .category
            .as_deref()
            .unwrap_or("General")
    }

    /// Primary database key: `config:<key>`.
    pub fn database_key(&self) -> String {
        format!("config:{}", self.key)
    }
}

/// Definition of a configuration subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsystemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub config_prefix: &'static str,
}

/// The standard subsystems of Portmaster.
pub const SUBSYSTEMS: &[SubsystemDef] = &[
    SubsystemDef {
        id: "filter",
        name: "Privacy Filter",
        description: "DNS & Network rules, filter lists, and blocking policies",
        config_prefix: "filter/",
    },
    SubsystemDef {
        id: "dns",
        name: "Secure DNS",
        description: "DNS-over-TLS, servers, and fallback resolvers",
        config_prefix: "dns/",
    },
    SubsystemDef {
        id: "xray",
        name: "VPN / Xray Gateway",
        description: "VLESS, Trojan, VMess proxy routing and split-tunneling",
        config_prefix: "xray/",
    },
    SubsystemDef {
        id: "splittun",
        name: "Split Tunnel",
        description: "Direct routing bypass and per-interface routing",
        config_prefix: "splittun/",
    },
    SubsystemDef {
        id: "history",
        name: "Network History",
        description: "Connection logging, retention, and Netquery database",
        config_prefix: "history/",
    },
    SubsystemDef {
        id: "core",
        name: "Core & System",
        description: "Base system integration, updates, UI, and logging",
        config_prefix: "core/",
    },
];
