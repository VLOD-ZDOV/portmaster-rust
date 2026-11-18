//! PortAPI WebSocket wire protocol and message representations.
//!
//! Frames on the wire follow the format:
//! `opID|type|key|data` or `opID|type|query` or `opID|type`
//! where record JSON data is prefixed with the DSD format byte `J`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Well-known WebSocket database paths in Portmaster.
pub const DB_API_PATH_DEFAULT: &str = "/api/database/v1";
pub const DB_API_PATH_ALT: &str = "/v1/database/ws";

/// Delimiter between wire frame parts.
pub const SEPARATOR: char = '|';
pub const SEPARATOR_BYTE: u8 = b'|';
/// Prefix byte for JSON encoded payload in DSD format.
pub const DSD_JSON_PREFIX: char = 'J';
pub const DSD_JSON_PREFIX_BYTE: u8 = b'J';

/// Request message types.
pub mod request_types {
    pub const GET: &str = "get";
    pub const QUERY: &str = "query";
    pub const SUB: &str = "sub";
    pub const QSUB: &str = "qsub";
    pub const CREATE: &str = "create";
    pub const UPDATE: &str = "update";
    pub const INSERT: &str = "insert";
    pub const DELETE: &str = "delete";
    pub const CANCEL: &str = "cancel";
}

/// Reply message types.
pub mod reply_types {
    pub const OK: &str = "ok";
    pub const UPD: &str = "upd";
    pub const NEW: &str = "new";
    pub const DEL: &str = "del";
    pub const SUCCESS: &str = "success";
    pub const ERROR: &str = "error";
    pub const WARNING: &str = "warning";
    pub const DONE: &str = "done";
}

/// Common record metadata present on all Portmaster database records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RecordMeta {
    #[serde(rename = "Created", default)]
    pub created: i64,
    #[serde(rename = "Deleted", default)]
    pub deleted: i64,
    #[serde(rename = "Expires", default)]
    pub expires: i64,
    #[serde(rename = "Modified", default)]
    pub modified: i64,
    #[serde(rename = "Key", default)]
    pub key: String,
}

/// A parsed PortAPI frame from WebSocket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortApiFrame {
    /// Multiplexing operation identifier.
    pub id: String,
    /// Message type (e.g. `ok`, `upd`, `new`, `del`, `done`, `success`, `error`, `warning`).
    pub msg_type: String,
    /// Database record key, query string, or error/warning message.
    pub key: Option<String>,
    /// Serialized JSON string payload (stripped of the leading `J` prefix).
    pub data: Option<String>,
}

impl PortApiFrame {
    /// Creates a new frame with the specified id and type.
    pub fn new(id: impl Into<String>, msg_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            msg_type: msg_type.into(),
            key: None,
            data: None,
        }
    }

    /// Deserializes a typed JSON value from `self.data`, if present.
    pub fn parse_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        let raw = self.data.as_deref().unwrap_or("null");
        serde_json::from_str(raw)
    }

    /// Parses a raw text frame from the WebSocket.
    pub fn parse(raw: &str) -> Result<Self, ProtocolError> {
        let parts: Vec<&str> = raw.splitn(4, SEPARATOR).collect();
        if parts.len() < 2 {
            return Err(ProtocolError::MalformedFrame(raw.to_string()));
        }

        let id = parts[0].to_string();
        let msg_type = parts[1].to_string();

        match msg_type.as_str() {
            reply_types::DONE | reply_types::SUCCESS | request_types::CANCEL => Ok(Self {
                id,
                msg_type,
                key: None,
                data: None,
            }),
            reply_types::ERROR | reply_types::WARNING => {
                let message = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    String::new()
                };
                Ok(Self {
                    id,
                    msg_type,
                    key: Some(message),
                    data: None,
                })
            }
            reply_types::DEL | request_types::GET | request_types::DELETE => {
                let key = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    String::new()
                };
                Ok(Self {
                    id,
                    msg_type,
                    key: Some(key),
                    data: None,
                })
            }
            request_types::QUERY | request_types::SUB | request_types::QSUB => {
                let mut query = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    String::new()
                };
                if query.starts_with("query ") {
                    query = query[6..].to_string();
                }
                Ok(Self {
                    id,
                    msg_type,
                    key: Some(query),
                    data: None,
                })
            }
            reply_types::OK
            | reply_types::UPD
            | reply_types::NEW
            | request_types::CREATE
            | request_types::UPDATE
            | request_types::INSERT => {
                let key = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    String::new()
                };
                let data = if parts.len() >= 4 {
                    let d = parts[3];
                    if let Some(stripped) = d.strip_prefix(DSD_JSON_PREFIX) {
                        Some(stripped.to_string())
                    } else {
                        Some(d.to_string())
                    }
                } else {
                    None
                };
                Ok(Self {
                    id,
                    msg_type,
                    key: Some(key),
                    data,
                })
            }
            _ => {
                // Generic fallback for custom / extended frame types
                let key = parts.get(2).map(|s| s.to_string());
                let data = parts.get(3).map(|s| {
                    if let Some(stripped) = s.strip_prefix(DSD_JSON_PREFIX) {
                        stripped.to_string()
                    } else {
                        s.to_string()
                    }
                });
                Ok(Self {
                    id,
                    msg_type,
                    key,
                    data,
                })
            }
        }
    }

    /// Serializes this frame into wire format: `id|type[|key[|Jdata]]`.
    pub fn to_wire(&self) -> String {
        match self.msg_type.as_str() {
            request_types::CANCEL | reply_types::DONE | reply_types::SUCCESS => {
                format!("{}{}{}", self.id, SEPARATOR, self.msg_type)
            }
            reply_types::ERROR | reply_types::WARNING => {
                let msg = self.key.as_deref().unwrap_or_default();
                format!("{}{}{}{}{}", self.id, SEPARATOR, self.msg_type, SEPARATOR, msg)
            }
            request_types::GET | request_types::DELETE | reply_types::DEL => {
                let k = self.key.as_deref().unwrap_or_default();
                format!("{}{}{}{}{}", self.id, SEPARATOR, self.msg_type, SEPARATOR, k)
            }
            request_types::QUERY | request_types::SUB | request_types::QSUB => {
                let q = self.key.as_deref().unwrap_or_default();
                let query_str = if q.starts_with("query ") {
                    q.to_string()
                } else {
                    format!("query {}", q)
                };
                format!(
                    "{}{}{}{}{}",
                    self.id, SEPARATOR, self.msg_type, SEPARATOR, query_str
                )
            }
            request_types::CREATE
            | request_types::UPDATE
            | request_types::INSERT
            | reply_types::OK
            | reply_types::UPD
            | reply_types::NEW => {
                let k = self.key.as_deref().unwrap_or_default();
                let d = self.data.as_deref().unwrap_or("{}");
                format!(
                    "{}{}{}{}{}{}{}{}",
                    self.id,
                    SEPARATOR,
                    self.msg_type,
                    SEPARATOR,
                    k,
                    SEPARATOR,
                    DSD_JSON_PREFIX,
                    d
                )
            }
            _ => {
                let mut out = format!("{}{}{}", self.id, SEPARATOR, self.msg_type);
                if let Some(k) = &self.key {
                    out.push(SEPARATOR);
                    out.push_str(k);
                    if let Some(d) = &self.data {
                        out.push(SEPARATOR);
                        out.push(DSD_JSON_PREFIX);
                        out.push_str(d);
                    }
                }
                out
            }
        }
    }
}

/// High-level request constructor.
#[derive(Debug, Clone, PartialEq)]
pub enum PortApiRequest {
    Get { id: String, key: String },
    Query { id: String, query: String },
    Sub { id: String, query: String },
    Qsub { id: String, query: String },
    Create { id: String, key: String, data: Value },
    Update { id: String, key: String, data: Value },
    Insert { id: String, key: String, data: Value },
    Delete { id: String, key: String },
    Cancel { id: String },
}

impl PortApiRequest {
    pub fn get(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Get {
            id: id.into(),
            key: key.into(),
        }
    }

    pub fn query(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self::Query {
            id: id.into(),
            query: query.into(),
        }
    }

    pub fn sub(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self::Sub {
            id: id.into(),
            query: query.into(),
        }
    }

    pub fn qsub(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self::Qsub {
            id: id.into(),
            query: query.into(),
        }
    }

    pub fn create(id: impl Into<String>, key: impl Into<String>, data: Value) -> Self {
        Self::Create {
            id: id.into(),
            key: key.into(),
            data,
        }
    }

    pub fn update(id: impl Into<String>, key: impl Into<String>, data: Value) -> Self {
        Self::Update {
            id: id.into(),
            key: key.into(),
            data,
        }
    }

    pub fn delete(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Delete {
            id: id.into(),
            key: key.into(),
        }
    }

    pub fn cancel(id: impl Into<String>) -> Self {
        Self::Cancel { id: id.into() }
    }

    /// Converts this request into its wire frame representation.
    pub fn to_frame(&self) -> PortApiFrame {
        match self {
            Self::Get { id, key } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::GET.to_string(),
                key: Some(key.clone()),
                data: None,
            },
            Self::Query { id, query } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::QUERY.to_string(),
                key: Some(query.clone()),
                data: None,
            },
            Self::Sub { id, query } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::SUB.to_string(),
                key: Some(query.clone()),
                data: None,
            },
            Self::Qsub { id, query } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::QSUB.to_string(),
                key: Some(query.clone()),
                data: None,
            },
            Self::Create { id, key, data } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::CREATE.to_string(),
                key: Some(key.clone()),
                data: Some(serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string())),
            },
            Self::Update { id, key, data } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::UPDATE.to_string(),
                key: Some(key.clone()),
                data: Some(serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string())),
            },
            Self::Insert { id, key, data } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::INSERT.to_string(),
                key: Some(key.clone()),
                data: Some(serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string())),
            },
            Self::Delete { id, key } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::DELETE.to_string(),
                key: Some(key.clone()),
                data: None,
            },
            Self::Cancel { id } => PortApiFrame {
                id: id.clone(),
                msg_type: request_types::CANCEL.to_string(),
                key: None,
                data: None,
            },
        }
    }

    /// Serializes directly to the wire string.
    pub fn to_wire(&self) -> String {
        self.to_frame().to_wire()
    }
}

/// Errors occurring during frame parsing or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    MalformedFrame(String),
    MissingPayload,
    JsonError(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedFrame(s) => write!(f, "malformed frame: {s}"),
            Self::MissingPayload => write!(f, "expected JSON payload missing in frame"),
            Self::JsonError(e) => write!(f, "JSON decoding error: {e}"),
        }
    }
}

impl std::error::Error for ProtocolError {}
