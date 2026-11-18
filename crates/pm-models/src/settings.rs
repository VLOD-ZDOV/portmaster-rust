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

    /// Primary database key: `config:options/<key>`.
    pub fn database_key(&self) -> String {
        format!("config:options/{}", self.key)
    }
}
