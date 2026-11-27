//! Xray integration for the Portmaster core.
//!
//! The Rust Xray core lives in a separate, MPL-2.0 licensed workspace and is
//! therefore **not** linked into this GPL-3.0 crate. Instead the `xray-rust`
//! binary is launched as a child process:
//!
//! ```text
//! xray-rust run -config <config.json>
//! ```
//!
//! The crate stores subscriptions in `pm-database`, fetches and parses their
//! profiles into [`model::XrayNode`]s (including countries for the SPN map),
//! generates an Xray JSON config, manages the child process and exposes a REST
//! API through `pm-api`.

pub mod api;
pub mod config;
pub mod error;
pub mod manager;
pub mod model;
pub mod parser;
pub mod stats;
pub mod subscription;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pm_config::{Config, ConfigOption, PossibleValue};
use pm_database::Databases;
use pm_geoip::GeoIp;
use serde_json::{json, Value};

pub use config::{InboundProtocol, DEFAULT_INBOUND_PORT};
pub use error::{Error, Result};
pub use manager::{ChildHandle, ProcessAdapter, SystemProcessAdapter, XrayManager};
pub use model::{
    AddSubscriptionRequest, ConnectRequest, NodeEditInfo, NodesResponse, ProcessState, Protocol,
    Subscription, TrafficStatus, UpdateNodeRequest, UpdateSubscriptionRequest, XrayNode,
    XrayStatus,
};
pub use stats::{
    GrpcResponse, GrpcStats, GrpcStatus, GrpcTransport, H2Transport, NoopTrafficStats, StatsError,
    StatsService, TrafficCounter, TrafficSnapshot, TrafficStats, DEFAULT_STATS_PORT, GET_STATS_PATH,
    QUERY_STATS_PATH,
};
pub use subscription::{HttpFetcher, SubscriptionFetcher, SubscriptionStore, XRAY_DB};

/// Path of the Xray binary in the configuration.
pub const CFG_XRAY_BINARY: &str = "core/xrayBinary";
/// Whether the Xray integration is enabled.
pub const CFG_XRAY_ENABLE: &str = "xray/enable";
/// Whether Xray automatically reconnects to the last server on startup.
pub const CFG_XRAY_AUTO_CONNECT: &str = "xray/autoConnect";
/// Local SOCKS/HTTP port exposed by Xray.
pub const CFG_XRAY_INBOUND_PORT: &str = "xray/inboundPort";
/// Local inbound protocol: `socks` or `http`.
pub const CFG_XRAY_INBOUND_PROTOCOL: &str = "xray/inboundProtocol";
/// Port of the local gRPC API inbound exposing Xray's StatsService. 0 disables it.
pub const CFG_XRAY_STATS_PORT: &str = "xray/statsPort";
/// Primary Xray binary setting under xray namespace.
pub const CFG_XRAY_BINARY_NAMESPACED: &str = "xray/binary";
/// Binary name looked up in the bin directory and PATH.
pub const XRAY_BINARY_NAME: &str = "xray-rust";

fn xray_binary_options() -> Vec<pm_config::PossibleValue> {
    #[cfg(windows)]
    {
        vec![
            pm_config::PossibleValue {
                name: "Auto-detect (%ProgramFiles%\\Portmaster, %ProgramData%\\Portmaster\\xray, PATH)".to_string(),
                description: "Автоматический поиск бинарника xray-rust.exe / xray.exe в каталоге Portmaster, стандартных путях Windows и PATH".to_string(),
                value: Value::String(String::new()),
            },
            pm_config::PossibleValue {
                name: "Portmaster Xray (%ProgramFiles%\\Portmaster\\xray-rust.exe)".to_string(),
                description: "Выделенный бинарник xray-rust.exe в Portmaster".to_string(),
                value: Value::String("xray-rust.exe".to_string()),
            },
            pm_config::PossibleValue {
                name: "v2rayN Binary (%ProgramFiles%\\v2rayN\\bin\\xray\\xray.exe)".to_string(),
                description: "Встроенный бинарник Xray core из v2rayN".to_string(),
                value: Value::String(r"C:\Program Files\v2rayN\bin\xray\xray.exe".to_string()),
            },
            pm_config::PossibleValue {
                name: "System binary (%ProgramFiles%\\Xray\\xray.exe)".to_string(),
                description: "Бинарник xray.exe в Program Files".to_string(),
                value: Value::String(r"C:\Program Files\Xray\xray.exe".to_string()),
            },
            pm_config::PossibleValue {
                name: "xray.exe (PATH)".to_string(),
                description: "Стандартный бинарник xray.exe из системного PATH".to_string(),
                value: Value::String("xray.exe".to_string()),
            },
        ]
    }
    #[cfg(not(windows))]
    {
        vec![
            pm_config::PossibleValue {
                name: "Auto-detect (поиск: <папка Portmaster>/xray-rust, /usr/bin, /usr/local/bin, /opt/xray-rust, PATH)".to_string(),
                description: "Автоматический поиск бинарника xray-rust в каталоге Portmaster, стандартных путях и PATH".to_string(),
                value: Value::String(String::new()),
            },
            pm_config::PossibleValue {
                name: "xray-rust (/usr/bin/xray-rust, /usr/local/bin/xray-rust, /opt/xray-rust/xray-rust)".to_string(),
                description: "Выделенный бинарник xray-rust".to_string(),
                value: Value::String("xray-rust".to_string()),
            },
            pm_config::PossibleValue {
                name: "v2rayN Binary (/opt/v2rayn-bin/bin/xray/xray)".to_string(),
                description: "Встроенный бинарник Xray core из v2rayN (/opt/v2rayn-bin/bin/xray/xray)".to_string(),
                value: Value::String("/opt/v2rayn-bin/bin/xray/xray".to_string()),
            },
            pm_config::PossibleValue {
                name: "System binary (/usr/bin/xray)".to_string(),
                description: "Системный бинарник /usr/bin/xray".to_string(),
                value: Value::String("/usr/bin/xray".to_string()),
            },
            pm_config::PossibleValue {
                name: "Local binary (/usr/local/bin/xray)".to_string(),
                description: "Локальный бинарник /usr/local/bin/xray".to_string(),
                value: Value::String("/usr/local/bin/xray".to_string()),
            },
            pm_config::PossibleValue {
                name: "xray (PATH)".to_string(),
                description: "Стандартный бинарник xray из системного PATH".to_string(),
                value: Value::String("xray".to_string()),
            },
        ]
    }
}

/// Registers the Xray configuration options.
pub fn register_options(config: &Config) -> pm_config::Result<()> {
    let default_binary = if cfg!(windows) {
        String::new()
    } else if Path::new("/opt/v2rayn-bin/bin/xray/xray").is_file() {
        "/opt/v2rayn-bin/bin/xray/xray".to_string()
    } else {
        String::new()
    };

    config.register(
        ConfigOption::new(
            "Модуль Xray",
            CFG_XRAY_ENABLE,
            "Включение интеграции и фонового процесса Xray для проксирования.",
            pm_config::OPT_TYPE_BOOL,
            Value::Bool(true),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_USER)
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(1))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("General")),
    )?;
    config.register(
        ConfigOption::new(
            "Автоподключение Xray",
            CFG_XRAY_AUTO_CONNECT,
            "Автоматически восстанавливать подключение к последнему серверу Xray при запуске Portmaster.",
            pm_config::OPT_TYPE_BOOL,
            Value::Bool(false),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_USER)
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(10))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("General")),
    )?;
    config.register(
        ConfigOption::new(
            "Xray Binary",
            CFG_XRAY_BINARY_NAMESPACED,
            "Path to the Xray binary. Select a pre-installed binary or auto-detect.",
            pm_config::OPT_TYPE_STRING,
            Value::String(default_binary),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_USER)
        .with_possible_values(xray_binary_options())
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(20))
        .with_annotation(pm_config::DISPLAY_HINT_ANNOTATION, json!(pm_config::DISPLAY_HINT_ONE_OF))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("General")),
    )?;
    config.register(
        ConfigOption::new(
            "Xray Local Port",
            CFG_XRAY_INBOUND_PORT,
            "Port of the local SOCKS/HTTP proxy exposed by Xray.",
            pm_config::OPT_TYPE_INT,
            Value::from(DEFAULT_INBOUND_PORT),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_DEVELOPER)
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(30))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("Development"))
        .with_validation_func(|value| match value.as_i64() {
            Some(port) if (1..=i64::from(u16::MAX)).contains(&port) => Ok(()),
            _ => Err("Xray local port must be between 1 and 65535".to_string()),
        }),
    )?;
    config.register(
        ConfigOption::new(
            "Xray Local Protocol",
            CFG_XRAY_INBOUND_PROTOCOL,
            "Local inbound protocol used by Xray: socks or http.",
            pm_config::OPT_TYPE_STRING,
            Value::String("socks".to_string()),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_DEVELOPER)
        .with_annotation(
            pm_config::DISPLAY_HINT_ANNOTATION,
            json!(pm_config::DISPLAY_HINT_ONE_OF),
        )
        .with_possible_values(vec![
            PossibleValue {
                name: "SOCKS5".to_string(),
                description: "SOCKS5 local proxy (TCP + UDP)".to_string(),
                value: Value::String("socks".to_string()),
            },
            PossibleValue {
                name: "HTTP".to_string(),
                description: "HTTP proxy (TCP only)".to_string(),
                value: Value::String("http".to_string()),
            },
        ])
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(40))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("Development")),
    )?;
    config.register(
        ConfigOption::new(
            "Xray Stats Port",
            CFG_XRAY_STATS_PORT,
            "Port of the local gRPC API inbound exposing Xray's StatsService. Set to 0 to disable traffic statistics.",
            pm_config::OPT_TYPE_INT,
            Value::from(i64::from(DEFAULT_STATS_PORT)),
        )
        .with_expertise_level(pm_config::EXPERTISE_LEVEL_DEVELOPER)
        .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(50))
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("Development")),
    )?;

    if config.option("spn/enable").is_err() {
        let _ = config.register(
            ConfigOption::new(
                "Состояние VPN (Xray)",
                "spn/enable",
                "Внутренний флаг активности VPN-подключения через Xray.",
                pm_config::OPT_TYPE_BOOL,
                Value::Bool(false),
            )
            .with_expertise_level(pm_config::EXPERTISE_LEVEL_DEVELOPER)
            .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("General")),
        );
    }

    if config.option("xray/rules").is_err() {
        let _ = config.register(
            ConfigOption::new(
                "Правила маршрутизации Xray",
                "xray/rules",
                "Правила для доменов, geoip/geosite категорий (например geoip:ru, geosite:category-ads-all), IP-сетей и портов: маршрутизировать через VPN (+) или пускать напрямую (-).",
                pm_config::OPT_TYPE_STRING_ARRAY,
                Value::Array(Vec::new()),
            )
            .with_expertise_level(pm_config::EXPERTISE_LEVEL_USER)
            .with_annotation(pm_config::DISPLAY_ORDER_ANNOTATION, json!(25))
            .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("General")),
        );
    }

    Ok(())
}

fn exists_in_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|directory| {
                if directory.join(name).is_file() {
                    return true;
                }
                #[cfg(windows)]
                {
                    if !name.to_lowercase().ends_with(".exe")
                        && directory.join(format!("{name}.exe")).is_file()
                    {
                        return true;
                    }
                }
                false
            })
        })
        .unwrap_or(false)
}

/// Resolves the Xray binary path: explicit config, then standard locations, then PATH.
/// On Windows, checks .exe extensions and standard installation directories.
/// Returns `None` when nothing is found.
pub fn xray_binary_path(configured: &str, data_dir: &Path, bin_dir: &Path) -> Option<String> {
    if !configured.is_empty() {
        let path = Path::new(configured);
        if path.is_file() {
            return Some(configured.to_string());
        }
        #[cfg(windows)]
        {
            if !configured.to_lowercase().ends_with(".exe") {
                let with_exe = format!("{configured}.exe");
                let path_exe = Path::new(&with_exe);
                if path_exe.is_file() {
                    return Some(with_exe);
                }
            }
        }
        if path.is_absolute() || configured.contains('/') || configured.contains('\\') {
            return None;
        }
        if exists_in_path(configured) {
            return Some(configured.to_string());
        }
        #[cfg(windows)]
        {
            if !configured.to_lowercase().ends_with(".exe") {
                let with_exe = format!("{configured}.exe");
                if exists_in_path(&with_exe) {
                    return Some(with_exe);
                }
            }
        }
        return None;
    }

    #[cfg(windows)]
    {
        // 1. bin_dir (xray-rust.exe, xray.exe, xray-rust, xray)
        let candidates_bin = [
            bin_dir.join(format!("{XRAY_BINARY_NAME}.exe")),
            bin_dir.join("xray.exe"),
            bin_dir.join(XRAY_BINARY_NAME),
            bin_dir.join("xray"),
        ];
        for c in &candidates_bin {
            if c.is_file() {
                return Some(c.to_string_lossy().into_owned());
            }
        }

        // 2. data_dir/xray (xray-rust.exe, xray.exe, xray-rust, xray)
        let xray_dir = data_dir.join("xray");
        let candidates_data = [
            xray_dir.join(format!("{XRAY_BINARY_NAME}.exe")),
            xray_dir.join("xray.exe"),
            xray_dir.join(XRAY_BINARY_NAME),
            xray_dir.join("xray"),
        ];
        for c in &candidates_data {
            if c.is_file() {
                return Some(c.to_string_lossy().into_owned());
            }
        }

        // 3. %ProgramFiles%\Portmaster\xray-rust.exe and xray.exe
        //    %ProgramFiles%\v2rayN\bin\xray\xray.exe
        //    %ProgramFiles%\Xray\xray.exe
        let mut program_files_dirs = Vec::new();
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            program_files_dirs.push(PathBuf::from(pf));
        }
        if let Some(pf) = std::env::var_os("ProgramW6432") {
            let p = PathBuf::from(pf);
            if !program_files_dirs.contains(&p) {
                program_files_dirs.push(p);
            }
        }
        if let Some(pf) = std::env::var_os("ProgramFiles(x86)") {
            let p = PathBuf::from(pf);
            if !program_files_dirs.contains(&p) {
                program_files_dirs.push(p);
            }
        }
        if program_files_dirs.is_empty() {
            program_files_dirs.push(PathBuf::from(r"C:\Program Files"));
        }

        for pf in &program_files_dirs {
            let candidates = [
                pf.join("Portmaster").join(format!("{XRAY_BINARY_NAME}.exe")),
                pf.join("Portmaster").join("xray.exe"),
                pf.join("v2rayN").join("bin").join("xray").join("xray.exe"),
                pf.join("Xray").join("xray.exe"),
            ];
            for c in &candidates {
                if c.is_file() {
                    return Some(c.to_string_lossy().into_owned());
                }
            }
        }

        // 4. %ProgramData%\Portmaster\xray\xray-rust.exe and xray.exe
        let mut program_data_dirs = Vec::new();
        if let Some(pd) = std::env::var_os("ProgramData") {
            program_data_dirs.push(PathBuf::from(pd));
        }
        if program_data_dirs.is_empty() {
            program_data_dirs.push(PathBuf::from(r"C:\ProgramData"));
        }

        for pd in &program_data_dirs {
            let candidates = [
                pd.join("Portmaster").join("xray").join(format!("{XRAY_BINARY_NAME}.exe")),
                pd.join("Portmaster").join("xray").join("xray.exe"),
                pd.join("Portmaster").join(format!("{XRAY_BINARY_NAME}.exe")),
                pd.join("Portmaster").join("xray.exe"),
            ];
            for c in &candidates {
                if c.is_file() {
                    return Some(c.to_string_lossy().into_owned());
                }
            }
        }

        // 5. PATH check with .exe
        let exe_names = [
            format!("{XRAY_BINARY_NAME}.exe"),
            "xray.exe".to_string(),
            XRAY_BINARY_NAME.to_string(),
            "xray".to_string(),
        ];
        for name in &exe_names {
            if exists_in_path(name) {
                return Some(name.clone());
            }
        }
    }

    #[cfg(not(windows))]
    {
        let xray_dir = data_dir.join("xray");
        let candidate_in_xray = xray_dir.join(XRAY_BINARY_NAME);
        if candidate_in_xray.is_file() {
            return Some(candidate_in_xray.to_string_lossy().into_owned());
        }
        let candidate_xray_in_xray = xray_dir.join("xray");
        if candidate_xray_in_xray.is_file() {
            return Some(candidate_xray_in_xray.to_string_lossy().into_owned());
        }
        let candidate = bin_dir.join(XRAY_BINARY_NAME);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
        let candidate_xray = bin_dir.join("xray");
        if candidate_xray.is_file() {
            return Some(candidate_xray.to_string_lossy().into_owned());
        }
        const KNOWN_PATHS: &[&str] = &[
            "/opt/v2rayn-bin/bin/xray/xray",
            "/usr/bin/xray",
            "/usr/local/bin/xray",
            "/opt/xray-rust/xray-rust",
            "/usr/bin/xray-rust",
            "/usr/local/bin/xray-rust",
        ];
        for &known in KNOWN_PATHS {
            if Path::new(known).is_file() {
                return Some(known.to_string());
            }
        }
        if exists_in_path("xray") {
            return Some("xray".to_string());
        }
        if exists_in_path(XRAY_BINARY_NAME) {
            return Some(XRAY_BINARY_NAME.to_string());
        }
    }

    None
}

/// Resolves the binary path: explicit config, then standard locations, then PATH.
pub fn resolve_binary(configured: &str, data_dir: &Path, bin_dir: &Path) -> Option<String> {
    xray_binary_path(configured, data_dir, bin_dir)
}

/// Current time as Unix seconds.
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// The Xray integration service, owned by the core.
pub struct Xray {
    pub config: Arc<Config>,
    pub store: SubscriptionStore,
    pub manager: XrayManager,
    pub geoip: Arc<GeoIp>,
    /// Source of traffic statistics: real gRPC client when a stats port is
    /// configured, [`NoopTrafficStats`] otherwise.
    pub stats: Arc<dyn StatsService>,
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
}

impl Xray {
    /// Creates the service. A missing or disabled binary is only a warning:
    /// construction never fails because of it.
    pub fn new(
        config: &Arc<Config>,
        databases: &Arc<Databases>,
        geoip: Arc<GeoIp>,
        data_dir: PathBuf,
        bin_dir: PathBuf,
    ) -> Arc<Xray> {
        let configured_binary = {
            let b = config.get_as_string(CFG_XRAY_BINARY_NAMESPACED, "")();
            if b.is_empty() {
                config.get_as_string(CFG_XRAY_BINARY, "")()
            } else {
                b
            }
        };
        let binary = resolve_binary(&configured_binary, &data_dir, &bin_dir).unwrap_or_default();
        let stats_port = config
            .get_as_int(CFG_XRAY_STATS_PORT, i64::from(DEFAULT_STATS_PORT))()
            .clamp(0, u16::MAX as i64) as u16;
        let stats: Arc<dyn StatsService> = if stats_port == 0 {
            Arc::new(NoopTrafficStats)
        } else {
            Arc::new(GrpcStats::new(stats_port))
        };

        Arc::new(Xray {
            config: Arc::clone(config),
            store: SubscriptionStore::new(databases)
                .expect("register xray database"),
            manager: XrayManager::system_with_stats_port(binary, data_dir.clone(), stats_port),
            geoip,
            stats,
            data_dir,
            bin_dir,
        })
    }

    /// Creates a service with an injectable process adapter, for tests.
    /// Stats fall back to the no-op source unless [`Xray::with_adapter_and_stats`] is used.
    pub fn with_adapter(
        config: &Arc<Config>,
        databases: &Arc<Databases>,
        geoip: Arc<GeoIp>,
        data_dir: PathBuf,
        binary: String,
        adapter: Box<dyn ProcessAdapter>,
    ) -> Arc<Xray> {
        Self::with_adapter_and_stats(
            config,
            databases,
            geoip,
            data_dir,
            binary,
            adapter,
            Arc::new(NoopTrafficStats),
        )
    }

    /// Like [`Xray::with_adapter`], with an injectable traffic stats source.
    pub fn with_adapter_and_stats(
        config: &Arc<Config>,
        databases: &Arc<Databases>,
        geoip: Arc<GeoIp>,
        data_dir: PathBuf,
        binary: String,
        adapter: Box<dyn ProcessAdapter>,
        stats: Arc<dyn StatsService>,
    ) -> Arc<Xray> {
        let bin_dir = data_dir.join("bin");
        Arc::new(Xray {
            config: Arc::clone(config),
            store: SubscriptionStore::new(databases)
                .expect("register xray database"),
            manager: XrayManager::with_stats_port(binary, data_dir.clone(), adapter, 0),
            geoip,
            stats,
            data_dir,
            bin_dir,
        })
    }

    /// Returns the SHA-256 digest GitHub publishes for a latest release asset.
    fn latest_asset_digest(agent: &ureq::Agent, repo: &str, asset_name: &str) -> Result<String> {
        use std::io::Read;
        let url = format!("https://api.github.com/repos/{repo}/releases/latest");
        let mut response = agent
            .get(&url)
            .header("User-Agent", "Portmaster-Rust")
            .header("Accept", "application/vnd.github+json")
            .call()
            .map_err(|e| Error::Fetch(format!("failed to fetch {repo} release metadata: {e}")))?;
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(2 * 1024 * 1024)
            .read_to_end(&mut body)
            .map_err(|e| Error::Fetch(format!("failed to read {repo} release metadata: {e}")))?;
        let release: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| Error::Fetch(format!("invalid {repo} release metadata: {e}")))?;
        let digest = release
            .get("assets")
            .and_then(|assets| assets.as_array())
            .and_then(|assets| {
                assets.iter().find(|asset| {
                    asset.get("name").and_then(|name| name.as_str()) == Some(asset_name)
                })
            })
            .and_then(|asset| asset.get("digest"))
            .and_then(|digest| digest.as_str())
            .ok_or_else(|| Error::Fetch(format!("no digest for {asset_name} in {repo}")))?;
        digest
            .strip_prefix("sha256:")
            .map(str::to_string)
            .ok_or_else(|| Error::Fetch(format!("unsupported digest for {asset_name}: {digest}")))
    }

    fn download_verified_asset(
        agent: &ureq::Agent,
        repo: &str,
        asset_name: &str,
        max_size: u64,
    ) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let expected_digest = Self::latest_asset_digest(agent, repo, asset_name)?;
        let download_url = format!("https://github.com/{repo}/releases/latest/download/{asset_name}");
        let mut response = agent
            .get(&download_url)
            .header("User-Agent", "Portmaster-Rust")
            .call()
            .map_err(|e| Error::Fetch(format!("failed to download {asset_name}: {e}")))?;

        let mut payload = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(max_size + 1)
            .read_to_end(&mut payload)
            .map_err(|e| Error::Process(format!("failed to read {asset_name} payload: {e}")))?;
        if payload.len() as u64 > max_size || payload.is_empty() {
            return Err(Error::Process(format!("{asset_name} archive/file size out of bounds")));
        }

        let actual = hex::encode(Sha256::digest(&payload));
        if !actual.eq_ignore_ascii_case(&expected_digest) {
            return Err(Error::Process(format!(
                "{asset_name} checksum mismatch: expected {expected_digest}, got {actual}"
            )));
        }
        Ok(payload)
    }

    /// Downloads the latest official Xray-core binary from GitHub, extracts it
    /// into `<data-dir>/xray`, marks it executable, and updates the manager.
    /// The upstream release asset is not signed; this is a separate trust path
    /// from the signed Portmaster update index.
    pub fn download_and_install_binary(&self) -> Result<String> {
        use std::io::Read;

        let (os, arch) = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => ("linux", "64"),
            ("linux", "aarch64") => ("linux", "arm64-v8a"),
            ("windows", "x86_64") => ("windows", "64"),
            ("windows", "aarch64") => ("windows", "arm64-v8a"),
            (other_os, other_arch) => {
                return Err(Error::Process(format!(
                    "unsupported OS/architecture for Xray download: {other_os} {other_arch}"
                )));
            }
        };

        let asset_name = format!("Xray-{os}-{arch}.zip");
        tracing::info!(%asset_name, "downloading Xray-core release binary");
        let agent = ureq::Agent::new_with_defaults();
        let zip_bytes = Self::download_verified_asset(&agent, "XTLS/Xray-core", &asset_name, 64 * 1024 * 1024)?;

        let cursor = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| Error::Process(format!("failed to parse zip archive: {e}")))?;

        let inner_binary_name = if cfg!(windows) { "xray.exe" } else { "xray" };
        let target_binary_name = if cfg!(windows) { "xray-rust.exe" } else { "xray-rust" };

        let mut binary_bytes: Option<Vec<u8>> = None;
        let mut geoip_bytes: Option<Vec<u8>> = None;
        let mut geosite_bytes: Option<Vec<u8>> = None;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| Error::Process(format!("failed to inspect zip entry: {e}")))?;
            let raw_name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };
            if raw_name == inner_binary_name || raw_name.ends_with(&format!("/{inner_binary_name}")) {
                if file.size() > 128 * 1024 * 1024 {
                    return Err(Error::Process("Xray binary inside archive is too large".to_string()));
                }
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|e| Error::Process(format!("failed to extract binary: {e}")))?;
                binary_bytes = Some(buf);
            } else if raw_name == "geoip.dat" || raw_name.ends_with("/geoip.dat") {
                if file.size() > 64 * 1024 * 1024 {
                    return Err(Error::Process("Xray geoip.dat is too large".to_string()));
                }
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|e| Error::Process(format!("failed to extract geoip.dat: {e}")))?;
                geoip_bytes = Some(buf);
            } else if raw_name == "geosite.dat" || raw_name.ends_with("/geosite.dat") {
                if file.size() > 64 * 1024 * 1024 {
                    return Err(Error::Process("Xray geosite.dat is too large".to_string()));
                }
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|e| Error::Process(format!("failed to extract geosite.dat: {e}")))?;
                geosite_bytes = Some(buf);
            }
        }

        let binary_data = binary_bytes.ok_or_else(|| {
            Error::Process(format!("'{inner_binary_name}' binary not found inside {asset_name}"))
        })?;

        // Determine destination: consolidate everything into data_dir/xray
        let xray_dir = self.data_dir.join("xray");
        std::fs::create_dir_all(&xray_dir)
            .map_err(|e| Error::Process(format!("failed to create dir {}: {e}", xray_dir.display())))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&xray_dir, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| Error::Process(format!("failed to secure xray dir: {e}")))?;
        }
        let final_path = xray_dir.join(target_binary_name);
        std::fs::write(&final_path, &binary_data)
            .map_err(|e| Error::Process(format!("failed to write binary {}: {e}", final_path.display())))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o755));
        }

        if let Some(geoip) = geoip_bytes {
            let _ = std::fs::write(xray_dir.join("geoip.dat"), geoip);
        }
        if let Some(geosite) = geosite_bytes {
            let _ = std::fs::write(xray_dir.join("geosite.dat"), geosite);
        }

        let path_str = final_path.to_string_lossy().into_owned();
        tracing::info!(path = %path_str, "Xray binary successfully installed");

        self.manager.set_binary(path_str.clone());
        let _ = self.config.set_config_option(CFG_XRAY_BINARY_NAMESPACED, serde_json::Value::String(path_str.clone()));
        let _ = self.config.set_config_option(CFG_XRAY_ENABLE, serde_json::Value::Bool(true));

        Ok(path_str)
    }

    /// Downloads the latest community `geoip.dat` and `geosite.dat` rulesets
    /// (`Loyalsoldier/v2ray-rules-dat`) into `<data-dir>/xray` with SHA-256 verification.
    pub fn download_and_install_geodata(&self) -> Result<(String, String)> {
        let xray_dir = self.data_dir.join("xray");
        std::fs::create_dir_all(&xray_dir)
            .map_err(|e| Error::Process(format!("failed to create dir {}: {e}", xray_dir.display())))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&xray_dir, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| Error::Process(format!("failed to secure xray dir: {e}")))?;
        }

        let agent = ureq::Agent::new_with_defaults();
        const MAX_DAT_SIZE: u64 = 64 * 1024 * 1024;
        let geoip_bytes = Self::download_verified_asset(
            &agent,
            "Loyalsoldier/v2ray-rules-dat",
            "geoip.dat",
            MAX_DAT_SIZE,
        )?;
        let geosite_bytes = Self::download_verified_asset(
            &agent,
            "Loyalsoldier/v2ray-rules-dat",
            "geosite.dat",
            MAX_DAT_SIZE,
        )?;

        let write_atomic = |name: &str, bytes: &[u8]| -> Result<String> {
            let final_path = xray_dir.join(name);
            let tmp_path = xray_dir.join(format!("{name}.tmp"));
            std::fs::write(&tmp_path, bytes)
                .map_err(|e| Error::Process(format!("failed to write {}: {e}", tmp_path.display())))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o644));
            }
            std::fs::rename(&tmp_path, &final_path)
                .map_err(|e| Error::Process(format!("failed to install {}: {e}", final_path.display())))?;
            Ok(final_path.to_string_lossy().into_owned())
        };

        let geoip_path = write_atomic("geoip.dat", &geoip_bytes)?;
        let geosite_path = write_atomic("geosite.dat", &geosite_bytes)?;
        let ts = now().to_string();
        let _ = std::fs::write(xray_dir.join(".geodata_updated_at"), ts.as_bytes());
        tracing::info!(%geoip_path, %geosite_path, "Xray GeoIP/GeoSite rulesets installed");
        Ok((geoip_path, geosite_path))
    }

    /// Refreshes `geoip.dat` / `geosite.dat` at most once per 24h alongside scheduled
    /// intelligence updates when Xray is enabled and geodata files or geo rules are used.
    pub fn maybe_update_geodata_if_due(self: &Arc<Self>) {
        if !self.is_enabled() {
            return;
        }
        let xray_dir = self.data_dir.join("xray");
        let has_geo_files = xray_dir.join("geoip.dat").is_file() || xray_dir.join("geosite.dat").is_file();
        let has_geo_rules = self
            .routing_rules()
            .iter()
            .any(|r| r.contains("geoip:") || r.contains("geosite:"));
        if !has_geo_files && !has_geo_rules {
            return;
        }
        let stamp_path = xray_dir.join(".geodata_updated_at");
        let last = std::fs::read_to_string(&stamp_path)
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .unwrap_or(0);
        if now().saturating_sub(last) < 86_400 {
            return;
        }
        if let Err(error) = self.download_and_install_geodata() {
            tracing::warn!(%error, "xray: scheduled geodata update failed");
            return;
        }
        if self.is_connected() {
            if let Err(error) = self.connect_last() {
                tracing::warn!(%error, "xray: failed to reload after scheduled geodata update");
            }
        }
    }

    /// Returns the process state enriched with live traffic counters. Stats
    /// are only queried for a running process; failures are logged and leave
    /// the `traffic` field unset so the status endpoint stays usable.
    pub fn status(&self) -> XrayStatus {
        let mut status = self.manager.status();
        if status.connected && self.stats.available() {
            match self.stats.traffic("proxy") {
                Ok(snapshot) => {
                    status.traffic = Some(TrafficStatus {
                        uplink: snapshot.uplink,
                        downlink: snapshot.downlink,
                    });
                }
                Err(error) => tracing::warn!(%error, "xray: stats unavailable"),
            }
        }
        status
    }

    /// Returns the PID of the active Xray child process, if running.
    pub fn active_pid(&self) -> Option<u32> {
        self.manager.active_pid()
    }

    /// Returns built-in routing presets for a node (`"ru-direct"`, `"bypass-ads"`, `"global-vpn"`).
    pub fn preset_rules(preset: &str) -> Vec<String> {
        match preset.trim().to_ascii_lowercase().as_str() {
            "ru-direct" | "ru_direct" => vec![
                "- geoip:ru".to_string(),
                "- geosite:category-ru".to_string(),
                "- geosite:category-ads-all".to_string(),
                "+ geosite:youtube".to_string(),
                "+ geosite:openai".to_string(),
                "+ geosite:discord".to_string(),
            ],
            "bypass-ads" | "ads" => vec!["- geosite:category-ads-all".to_string()],
            "global-vpn" | "global" => Vec::new(),
            _ => Vec::new(),
        }
    }

    /// Returns configured Xray-internal routing rules (`xray/rules`, e.g. `geosite:`/`geoip:`).
    /// Per-app and firewall endpoint rules (`spn/usagePolicy`) are enforced by the firewall directly.
    pub fn routing_rules(&self) -> Vec<String> {
        let mut rules = self.config.get_as_string_array("xray/rules", &[])();
        // Always ensure private LAN subnets and local addresses are bypassed directly.
        if !rules.iter().any(|r| r.contains("LAN") || r.contains("geoip:private") || r.contains("192.168.")) {
            rules.insert(0, "- LAN".to_string());
            rules.insert(1, "- Localhost".to_string());
        }
        rules
    }

    /// Merges per-node routing rules + preset + global `xray/rules`, keeping `- LAN`/`- Localhost` first.
    pub fn routing_rules_for_node(&self, node: &XrayNode) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut push_unique = |rule: String| {
            let trimmed = rule.trim().to_string();
            if !trimmed.is_empty() && seen.insert(trimmed.clone()) {
                out.push(trimmed);
            }
        };
        push_unique("- LAN".to_string());
        push_unique("- Localhost".to_string());
        for rule in &node.routing_rules {
            push_unique(rule.clone());
        }
        for rule in Self::preset_rules(&node.rule_preset) {
            push_unique(rule);
        }
        if node.rule_preset.trim() != "global-vpn" {
            for rule in self.routing_rules() {
                push_unique(rule);
            }
        }
        out
    }

    /// Returns editable structural metadata for a stored node (credentials remain masked).
    pub fn node_edit_info(&self, id: &str) -> Result<NodeEditInfo> {
        let node = self.store.get_node(id)?;
        let parsed = parser::parse_link(&node.raw_config)
            .map_err(|e| crate::Error::InvalidRequest(e.to_string()))?;
        Ok(NodeEditInfo {
            id: node.id,
            subscription_id: node.subscription_id,
            name: node.name,
            country: node.country_code,
            country_name: node.country_name,
            protocol: parsed.protocol.as_str().to_string(),
            address: parsed.address,
            port: parsed.port,
            method: parsed.method,
            network: parsed.network,
            security: parsed.security,
            path: parsed.path,
            host: parsed.host,
            sni: parsed.sni,
            fingerprint: parsed.fingerprint,
            alpn: parsed.alpn,
            flow: parsed.flow,
            service_name: parsed.service_name,
            header_type: parsed.header_type,
            reality_public_key: parsed.reality_public_key,
            reality_short_id: parsed.reality_short_id,
            reality_spider_x: parsed.reality_spider_x,
            routing_rules: node.routing_rules,
            rule_preset: node.rule_preset,
            has_credential: !parsed.credential.is_empty() || !parsed.password.is_empty(),
        })
    }

    /// Updates editable fields, share link, or per-node routing rules for a stored node.
    /// Re-encodes `raw_config`, preserves existing secrets when omitted, and reconnects if active.
    pub fn update_node(self: &Arc<Self>, id: &str, req: UpdateNodeRequest) -> Result<XrayNode> {
        let existing = self.store.get_node(id)?;
        let mut parsed = if let Some(link) = req.share_link.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parser::parse_link(link).map_err(|e| crate::Error::InvalidRequest(e.to_string()))?
        } else {
            parser::parse_link(&existing.raw_config)
                .map_err(|e| crate::Error::InvalidRequest(e.to_string()))?
        };

        if let Some(name) = req.name {
            parsed.name = name.trim().to_string();
        }
        if let Some(proto) = req.protocol {
            let p = Protocol::parse(&proto);
            if p != Protocol::Unknown {
                parsed.protocol = p;
            }
        }
        if let Some(addr) = req.address {
            let trimmed = addr.trim().to_string();
            if trimmed.is_empty() {
                return Err(crate::Error::InvalidRequest("address must not be empty".into()));
            }
            parsed.address = trimmed;
        }
        if let Some(port) = req.port {
            if port == 0 {
                return Err(crate::Error::InvalidRequest("port must be between 1 and 65535".into()));
            }
            parsed.port = port;
        }
        if let Some(cred) = req.credential.filter(|s| !s.trim().is_empty()) {
            parsed.credential = cred.trim().to_string();
        }
        if let Some(pass) = req.password.filter(|s| !s.trim().is_empty()) {
            parsed.password = pass.trim().to_string();
        }
        if let Some(method) = req.method {
            parsed.method = method.trim().to_string();
        }
        if let Some(net) = req.network {
            parsed.network = net.trim().to_string();
        }
        if let Some(sec) = req.security {
            parsed.security = sec.trim().to_string();
        }
        if let Some(path) = req.path {
            parsed.path = path.trim().to_string();
        }
        if let Some(host) = req.host {
            parsed.host = host.trim().to_string();
        }
        if let Some(sni) = req.sni {
            parsed.sni = sni.trim().to_string();
        }
        if let Some(fp) = req.fingerprint {
            parsed.fingerprint = fp.trim().to_string();
        }
        if let Some(alpn) = req.alpn {
            parsed.alpn = alpn.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(flow) = req.flow {
            parsed.flow = flow.trim().to_string();
        }
        if let Some(svc) = req.service_name {
            parsed.service_name = svc.trim().to_string();
        }
        if let Some(ht) = req.header_type {
            parsed.header_type = ht.trim().to_string();
        }
        if let Some(pbk) = req.reality_public_key {
            parsed.reality_public_key = pbk.trim().to_string();
        }
        if let Some(sid) = req.reality_short_id {
            parsed.reality_short_id = sid.trim().to_string();
        }
        if let Some(spx) = req.reality_spider_x {
            parsed.reality_spider_x = spx.trim().to_string();
        }

        let new_link = parser::encode_share_link(&parsed);
        let mut updated = parser::parse_profile(&new_link, &existing.subscription_id, Some(&self.geoip))
            .map_err(|e| crate::Error::InvalidRequest(e.to_string()))?;
        updated.id = existing.id.clone();
        updated.routing_rules = req.routing_rules.unwrap_or(existing.routing_rules);
        updated.rule_preset = req.rule_preset.unwrap_or(existing.rule_preset);

        self.store.save_node(&updated)?;
        if self.store.active_node_id().as_deref() == Some(id) && self.is_connected() {
            let _ = self.connect_node(id, None);
        }
        Ok(updated)
    }

    /// Removes a single node by ID, disconnecting first if it is currently active.
    pub fn delete_node(&self, id: &str) -> Result<()> {
        if self.store.active_node_id().as_deref() == Some(id) && self.is_connected() {
            let _ = self.disconnect();
        }
        self.store.remove_node(id)
    }

    /// Returns GeoIP country codes and GeoSite category hints from downloaded `.dat` files
    /// (with curated fallback suggestions when geodata has not yet been downloaded).
    pub fn geodata_hints(&self) -> Value {
        let xray_dir = self.data_dir.join("xray");
        let geoip_path = xray_dir.join("geoip.dat");
        let geosite_path = xray_dir.join("geosite.dat");
        let geoip_avail = geoip_path.is_file();
        let geosite_avail = geosite_path.is_file();

        let mut geoip_codes = if geoip_avail {
            extract_dat_categories(&geoip_path, 250)
        } else {
            Vec::new()
        };
        let mut geosite_cats = if geosite_avail {
            extract_dat_categories(&geosite_path, 400)
        } else {
            Vec::new()
        };

        for fallback in ["ru", "us", "de", "nl", "fi", "gb", "fr", "jp", "sg", "tr", "private"] {
            if !geoip_codes.iter().any(|c| c.eq_ignore_ascii_case(fallback)) {
                geoip_codes.push(fallback.to_string());
            }
        }
        for fallback in [
            "category-ads-all",
            "category-ru",
            "youtube",
            "openai",
            "google",
            "discord",
            "telegram",
            "github",
            "steam",
            "twitch",
            "instagram",
            "netflix",
        ] {
            if !geosite_cats.iter().any(|c| c.eq_ignore_ascii_case(fallback)) {
                geosite_cats.push(fallback.to_string());
            }
        }

        json!({
            "geoipAvailable": geoip_avail,
            "geositeAvailable": geosite_avail,
            "geoipCodes": geoip_codes,
            "geositeCategories": geosite_cats,
            "presets": [
                { "id": "default", "name": "По умолчанию (глобальные правила)" },
                { "id": "ru-direct", "name": "РФ напрямую + YouTube/AI/Discord в VPN" },
                { "id": "bypass-ads", "name": "Блокировка рекламы (category-ads-all)" },
                { "id": "global-vpn", "name": "Весь интернет-трафик через VPN" }
            ]
        })
    }

    /// Refreshes subscriptions and returns the number of discovered nodes.
    /// Fetch errors are logged and reported as zero; they never abort startup.
    pub fn refresh(&self, id: &str, fetcher: Option<&dyn SubscriptionFetcher>) -> usize {
        let fallback = HttpFetcher::new();
        let fetcher = fetcher.unwrap_or(&fallback);
        match self.store.refresh(id, fetcher, Some(&self.geoip)) {
            Ok(count) => {
                // replace_nodes clears the active pointer when the selected
                // node disappeared. Stop the old process too instead of
                // keeping a stale connection with no selectable node.
                if self.is_connected() && self.store.active_node_id().is_none() {
                    tracing::warn!("xray: active node disappeared during subscription refresh");
                    let _ = self.manager.disconnect();
                    self.reset_vpn_config();
                }
                count
            }
            Err(error) => {
                tracing::warn!(%error, "xray: subscription refresh failed");
                0
            }
        }
    }

    /// Returns true if Xray is currently running/connected.
    pub fn is_connected(&self) -> bool {
        self.manager.status().connected
    }

    /// Returns the running child's inbound port/protocol.
    pub fn active_inbound(&self) -> Option<(u16, InboundProtocol)> {
        self.manager.active_inbound()
    }

    fn stale_xray_proxy_url(&self, port: u16) -> String {
        let scheme = if matches!(self.configured_inbound(), InboundProtocol::Http) {
            "http"
        } else {
            "socks5"
        };
        format!("{scheme}://127.0.0.1:{port}")
    }

    fn clear_stale_splittun_proxy(&self, port: u16) {
        let xray_proxy = self.stale_xray_proxy_url(port);
        if self.config.get_as_string("splittun/proxy", "")() == xray_proxy {
            let _ = self.config.set_config_option(
                "splittun/proxy",
                serde_json::Value::String(String::new()),
            );
        }
    }

    /// Clears VPN state only. Split Tunnel settings are intentionally left
    /// untouched, so disconnecting Xray does not disable Split Tunnel.
    pub(crate) fn reset_vpn_config(&self) {
        let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
        let port = self.configured_inbound_port();
        self.clear_stale_splittun_proxy(port);
    }

    /// Returns whether the Xray integration is enabled in the current config.
    pub fn is_enabled(&self) -> bool {
        self.config.get_as_bool(CFG_XRAY_ENABLE, true)()
    }

    /// Returns the configured local inbound port, clamped to a valid u16.
    pub fn configured_inbound_port(&self) -> u16 {
        self.config
            .get_as_int(CFG_XRAY_INBOUND_PORT, i64::from(DEFAULT_INBOUND_PORT))()
            .clamp(1, u16::MAX as i64) as u16
    }

    /// Returns the configured local inbound protocol.
    pub fn configured_inbound(&self) -> InboundProtocol {
        if self
            .config
            .get_as_string(CFG_XRAY_INBOUND_PROTOCOL, "socks")()
            .eq_ignore_ascii_case("http")
        {
            InboundProtocol::Http
        } else {
            InboundProtocol::Socks
        }
    }

    /// Connects to a specific node ID using stored routing rules and optional inbound port.
    pub fn connect_node(
        self: &Arc<Self>,
        node_id: &str,
        inbound_port_override: Option<u16>,
    ) -> Result<XrayStatus> {
        if !self.is_enabled() {
            self.reset_vpn_config();
            return Err(crate::Error::Disabled);
        }
        if !self.manager.binary_available() {
            self.reset_vpn_config();
            return Err(crate::Error::Process(format!("binary not found: {XRAY_BINARY_NAME}")));
        }

        let node = match self.store.get_node(node_id) {
            Ok(node) => node,
            Err(error) => {
                // If connecting the stored last node failed because it was removed,
                // clear stale state so spn/enable does not stay true.
                if self.store.last_node_id().as_deref() == Some(node_id) {
                    self.store.clear_last_node();
                }
                self.store.clear_active();
                self.reset_vpn_config();
                return Err(error);
            }
        };

        let inbound_port = inbound_port_override.unwrap_or_else(|| self.configured_inbound_port());
        tracing::info!(node = %node.name, "xray: connecting to node");
        let rules = self.routing_rules_for_node(&node);
        let custom_rules = if rules.is_empty() { None } else { Some(rules.as_slice()) };
        if let Err(error) = self.manager.connect_with_rules(
            &node,
            inbound_port,
            self.configured_inbound(),
            custom_rules,
        ) {
            self.store.clear_active();
            self.reset_vpn_config();
            return Err(error);
        }

        self.store.set_active(&node.subscription_id, &node.id)?;
        if inbound_port_override.is_some() {
            let _ = self.config.set_config_option(
                CFG_XRAY_INBOUND_PORT,
                serde_json::Value::Number(u64::from(inbound_port).into()),
            );
        }
        let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(true));
        self.clear_stale_splittun_proxy(inbound_port);

        let probe = Arc::clone(self);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            probe.manager.probe_exit_ip(Some(&probe.geoip));
        });
        Ok(self.manager.status())
    }

    /// Connects to the last selected/active node using stored routing rules and inbound settings.
    pub fn connect_last(self: &Arc<Self>) -> Result<XrayStatus> {
        if !self.is_enabled() {
            self.reset_vpn_config();
            return Err(crate::Error::Disabled);
        }
        let node_id = self
            .store
            .last_node_id()
            .ok_or_else(|| {
                self.reset_vpn_config();
                crate::Error::Process("no previous active node found to connect".into())
            })?;
        self.connect_node(&node_id, None)
    }

    /// Disconnects Xray and resets proxy/spn config while preserving last_node_id in database.
    pub fn disconnect(&self) -> XrayStatus {
        let _ = self.manager.disconnect();
        self.store.clear_active();
        self.reset_vpn_config();
        self.manager.status()
    }

    /// Toggles Xray on or off. If connected, disconnects; if disconnected, connects to last node.
    pub fn toggle(self: &Arc<Self>) -> Result<XrayStatus> {
        if self.is_connected() {
            Ok(self.disconnect())
        } else {
            self.connect_last()
        }
    }

    /// Starts the service: restores the last selected node when one exists and auto-connect is enabled.
    pub fn start(self: &Arc<Self>) {
        if !self.is_enabled() {
            tracing::info!("xray: integration is disabled in settings");
            self.reset_vpn_config();
            return;
        }
        if !self.manager.binary_available() {
            tracing::warn!(
                binary = XRAY_BINARY_NAME,
                "xray: binary not found; integration stays inactive"
            );
            self.reset_vpn_config();
            return;
        }
        let auto_connect = self.config.get_as_bool(CFG_XRAY_AUTO_CONNECT, false)();
        if !auto_connect {
            tracing::info!("xray: auto-connect is disabled in settings");
            self.reset_vpn_config();
            return;
        }
        if let Err(error) = self.connect_last() {
            tracing::info!(%error, "xray: auto-connect skipped or no previous active node");
        }
    }

    /// Stops the child process; called when the core shuts down.
    pub fn stop(&self) {
        self.manager.stop();
        self.reset_vpn_config();
    }
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Option<usize> {
    let mut val = 0usize;
    let mut shift = 0u32;
    while *pos < buf.len() {
        let b = buf[*pos];
        *pos += 1;
        val |= ((b & 0x7f) as usize) << shift;
        if (b & 0x80) == 0 {
            return Some(val);
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }
    None
}

/// Scans an Xray protobuf `.dat` list (`GeoIPList` / `GeoSiteList`) and extracts
/// field #1 category/country codes without loading full IP/domain tries.
fn extract_dat_categories(path: &Path, limit: usize) -> Vec<String> {
    let Ok(data) = std::fs::read(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut pos = 0usize;
    while pos < data.len() && out.len() < limit {
        let Some(tag) = read_varint(&data, &mut pos) else {
            break;
        };
        let wire = tag & 0x7;
        let field = tag >> 3;
        match wire {
            0 => {
                if read_varint(&data, &mut pos).is_none() {
                    break;
                }
            }
            1 => {
                if pos.checked_add(8).is_some_and(|e| e <= data.len()) {
                    pos += 8;
                } else {
                    break;
                }
            }
            2 => {
                let Some(len) = read_varint(&data, &mut pos) else {
                    break;
                };
                let Some(end) = pos.checked_add(len).filter(|e| *e <= data.len()) else {
                    break;
                };
                if field == 1 {
                    let msg = &data[pos..end];
                    let mut mp = 0usize;
                    if let Some(mtag) = read_varint(msg, &mut mp) {
                        if mtag == 0x0a {
                            if let Some(slen) = read_varint(msg, &mut mp) {
                                if let Some(send) = mp.checked_add(slen).filter(|e| *e <= msg.len()) {
                                    if let Ok(code) = std::str::from_utf8(&msg[mp..send]) {
                                        let lower = code.trim().to_ascii_lowercase();
                                        if !lower.is_empty()
                                            && lower.len() <= 64
                                            && lower.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                                            && seen.insert(lower.clone())
                                        {
                                            out.push(lower);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                pos = end;
            }
            5 => {
                if pos.checked_add(4).is_some_and(|e| e <= data.len()) {
                    pos += 4;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_explicit_and_missing_binary() {
        let temp = std::env::temp_dir();
        assert!(xray_binary_path("/non/existent/path/xray", &temp, &temp).is_none());

        let dummy_exe = temp.join(format!("dummy-test-{}.exe", std::process::id()));
        std::fs::write(&dummy_exe, b"").unwrap();
        let path_str = dummy_exe.to_string_lossy().into_owned();
        assert_eq!(
            xray_binary_path(&path_str, &temp, &temp),
            Some(path_str.clone())
        );
        let _ = std::fs::remove_file(&dummy_exe);
    }

    #[test]
    fn resolves_binary_in_data_or_bin_dir() {
        let temp = std::env::temp_dir().join(format!("xray-find-test-{}", std::process::id()));
        let bin_dir = temp.join("bin");
        let data_dir = temp.join("data");
        let xray_data = data_dir.join("xray");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::create_dir_all(&xray_data).unwrap();

        let target_name = if cfg!(windows) { "xray-rust.exe" } else { "xray-rust" };
        let dummy = xray_data.join(target_name);
        std::fs::write(&dummy, b"").unwrap();

        let found = xray_binary_path("", &data_dir, &bin_dir);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with(target_name));

        let _ = std::fs::remove_dir_all(&temp);
    }
}
