//! Pure data models and wire protocol types for Portmaster PortAPI and Leptos UI.
//!
//! This crate has zero native system dependencies and is fully compatible with
//! both native targets and `wasm32-unknown-unknown`.

pub mod profile;
pub mod protocol;
pub mod settings;
pub mod xray;

pub use profile::{
    AppProfile, AppStats, FingerprintOperation, FingerprintType, ProfileFingerprint, ProfileIcon,
};
pub use protocol::{
    reply_types, request_types, PortApiFrame, PortApiRequest, ProtocolError, RecordMeta,
    DB_API_PATH_ALT, DB_API_PATH_DEFAULT, DSD_JSON_PREFIX, SEPARATOR,
};
pub use settings::{
    ExpertiseLevel, OptionType, PossibleValue, QuickSetting, ReleaseLevel, Setting,
    SettingAnnotations, SubsystemDef, SUBSYSTEMS,
};
pub use xray::{
    ConnectRequest, ProcessState, TrafficStatus, XrayNode, XrayStatus, XraySubscription,
};
