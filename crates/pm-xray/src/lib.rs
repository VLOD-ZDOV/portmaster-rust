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

use pm_config::{Config, ConfigOption};
use pm_database::Databases;
use pm_geoip::GeoIp;
use serde_json::{json, Value};

pub use config::{InboundProtocol, DEFAULT_INBOUND_PORT};
pub use error::{Error, Result};
pub use manager::{ChildHandle, ProcessAdapter, SystemProcessAdapter, XrayManager};
pub use model::{
    AddSubscriptionRequest, ConnectRequest, NodesResponse, ProcessState, Protocol, Subscription,
    TrafficStatus, UpdateSubscriptionRequest, XrayNode, XrayStatus,
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
            Value::Bool(true),
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
        .with_annotation(pm_config::CATEGORY_ANNOTATION, json!("Development")),
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
    pub enabled: bool,
    pub inbound_port: u16,
    pub inbound: InboundProtocol,
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
        let enabled = config.get_as_bool(CFG_XRAY_ENABLE, false)();
        let configured_binary = {
            let b = config.get_as_string(CFG_XRAY_BINARY_NAMESPACED, "")();
            if b.is_empty() {
                config.get_as_string(CFG_XRAY_BINARY, "")()
            } else {
                b
            }
        };
        let binary = resolve_binary(&configured_binary, &data_dir, &bin_dir).unwrap_or_default();
        let inbound_port = config
            .get_as_int(CFG_XRAY_INBOUND_PORT, i64::from(DEFAULT_INBOUND_PORT))()
            .clamp(1, u16::MAX as i64) as u16;
        let inbound = if config
            .get_as_string(CFG_XRAY_INBOUND_PROTOCOL, "socks")()
            .eq_ignore_ascii_case("http")
        {
            InboundProtocol::Http
        } else {
            InboundProtocol::Socks
        };
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
            enabled,
            inbound_port,
            inbound,
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
            enabled: true,
            inbound_port: DEFAULT_INBOUND_PORT,
            inbound: InboundProtocol::Socks,
            stats,
            data_dir,
            bin_dir,
        })
    }

    /// Downloads the latest official Xray-core binary from GitHub, extracts it into
    /// the Portmaster bin or data directory, marks it executable, and updates the manager.
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
        let download_url = format!("https://github.com/XTLS/Xray-core/releases/latest/download/{asset_name}");

        tracing::info!(%download_url, "downloading Xray-core release binary");
        let agent = ureq::Agent::new_with_defaults();
        let mut response = agent
            .get(&download_url)
            .header("User-Agent", "Portmaster-Rust")
            .call()
            .map_err(|e| Error::Fetch(format!("failed to download {asset_name}: {e}")))?;

        let mut zip_bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .read_to_end(&mut zip_bytes)
            .map_err(|e| Error::Process(format!("failed to read zip payload: {e}")))?;

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
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|e| Error::Process(format!("failed to extract binary: {e}")))?;
                binary_bytes = Some(buf);
            } else if raw_name == "geoip.dat" || raw_name.ends_with("/geoip.dat") {
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf);
                geoip_bytes = Some(buf);
            } else if raw_name == "geosite.dat" || raw_name.ends_with("/geosite.dat") {
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf);
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

    /// Returns configured Xray routing rules (merging `xray/rules` and `spn/usagePolicy`).
    pub fn routing_rules(&self) -> Vec<String> {
        let mut rules = self.config.get_as_string_array("xray/rules", &[])();
        if rules.is_empty() {
            rules = self.config.get_as_string_array("spn/usagePolicy", &[])();
        }
        // Always ensure private LAN subnets and local domains are bypassed directly
        if !rules.iter().any(|r| r.contains("LAN") || r.contains("geoip:private") || r.contains("192.168.")) {
            rules.insert(0, "- LAN".to_string());
            rules.insert(1, "- Localhost".to_string());
        }
        rules
    }

    /// Refreshes subscriptions and returns the number of discovered nodes.
    /// Fetch errors are logged and reported as zero; they never abort startup.
    pub fn refresh(&self, id: &str, fetcher: Option<&dyn SubscriptionFetcher>) -> usize {
        let fallback = HttpFetcher::new();
        let fetcher = fetcher.unwrap_or(&fallback);
        match self.store.refresh(id, fetcher, Some(&self.geoip)) {
            Ok(count) => count,
            Err(error) => {
                tracing::warn!(%error, "xray: subscription refresh failed");
                0
            }
        }
    }

    /// Starts the service: restores the last selected node when one exists and auto-connect is enabled.
    pub fn start(&self) {
        if !self.manager.binary_available() {
            tracing::warn!(
                binary = XRAY_BINARY_NAME,
                "xray: binary not found; integration stays inactive"
            );
            let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
            return;
        }
        let auto_connect = self.config.get_as_bool(CFG_XRAY_AUTO_CONNECT, true)();
        if !auto_connect {
            tracing::info!("xray: auto-connect is disabled in settings");
            let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
            return;
        }
        let Some(node_id) = self.store.last_node_id() else {
            tracing::info!("xray: no previous active node found to restore");
            let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
            let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
            return;
        };
        match self.store.get_node(&node_id) {
            Ok(node) => {
                tracing::info!(node = %node.name, "xray: auto-connecting to last selected node");
                let rules = self.routing_rules();
                let custom_rules = if rules.is_empty() { None } else { Some(rules.as_slice()) };
                if let Err(error) = self.manager.connect_with_rules(&node, self.inbound_port, self.inbound, custom_rules) {
                    tracing::warn!(%error, "xray: failed to restore selected node");
                    let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
                    let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
                    let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
                } else {
                    let _ = self.store.set_active(&node.subscription_id, &node.id);
                    let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(true));
                    let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(true));
                    let _ = self.config.set_config_option(
                        "splittun/proxy",
                        serde_json::Value::String(format!("socks5://127.0.0.1:{}", self.inbound_port)),
                    );
                }
            }
            Err(error) => {
                tracing::warn!(%error, "xray: stored active node is gone");
                let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
                let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
                let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
            }
        }
    }

    /// Stops the child process; called when the core shuts down.
    pub fn stop(&self) {
        self.manager.stop();
        let _ = self.config.set_config_option("spn/enable", serde_json::Value::Bool(false));
        let _ = self.config.set_config_option("splittun/enable", serde_json::Value::Bool(false));
        let _ = self.config.set_config_option("splittun/proxy", serde_json::Value::String(String::new()));
    }
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
