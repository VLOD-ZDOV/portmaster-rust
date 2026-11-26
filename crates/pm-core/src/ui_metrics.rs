//! Live UI flavor inventory, memory telemetry (RSS/PSS), and web-port status.

#[cfg(target_os = "linux")]
use std::collections::HashSet;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::sync::{Mutex, OnceLock};
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

use pm_api::{Api, Endpoint, PERMIT_ADMIN, PERMIT_USER};
use pm_config::Config;
use serde::Serialize;
use serde_json::json;

use crate::service::{CFG_KEEP_API_PORT_OPEN, CFG_UI_FLAVOR};
use crate::ui::{find_qt_binary, find_rust_ui_dir, ApiPortController};

/// Known benchmark baselines measured cleanly on a Linux workstation.
const ANGULAR_BENCHMARK_TAB_MB: f64 = 142.2;
const ANGULAR_BENCHMARK_TAURI_MB: f64 = 603.2;
const RUST_WASM_BENCHMARK_TAB_MB: f64 = 73.6;
const QT_BENCHMARK_MB: f64 = 140.6;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiFlavorInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub installed: bool,
    pub running: bool,
    pub active: bool,
    pub live_ram_mb: Option<f64>,
    pub live_rss_mb: Option<f64>,
    pub benchmark_ram_mb: f64,
    pub browser_tab_ram_mb: Option<f64>,
    pub process_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMemoryInfo {
    pub live_rss_mb: Option<f64>,
    pub live_pss_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiFlavorsReport {
    pub active_flavor: String,
    pub keep_api_port_open: bool,
    pub api_port_listening: bool,
    pub api_port_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unix_socket_path: Option<String>,
    pub flavors: Vec<UiFlavorInfo>,
    pub core_memory: ProcessMemoryInfo,
    pub xray_memory: ProcessMemoryInfo,
}

#[derive(Default, Clone)]
struct LiveStats {
    rss_kb: u64,
    pss_kb: u64,
    has_pss: bool,
    count: usize,
}

impl LiveStats {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn add(&mut self, rss_kb: u64, pss_kb: Option<u64>) {
        self.rss_kb = self.rss_kb.saturating_add(rss_kb);
        if let Some(pss) = pss_kb {
            self.pss_kb = self.pss_kb.saturating_add(pss);
            self.has_pss = true;
        }
        self.count += 1;
    }

    fn effective_mb(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        let kb = if self.has_pss { self.pss_kb } else { self.rss_kb };
        Some(round1(kb as f64 / 1024.0))
    }

    fn rss_mb(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        Some(round1(self.rss_kb as f64 / 1024.0))
    }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[cfg(target_os = "linux")]
fn read_proc_rss_pss(pid: u32) -> Option<(u64, Option<u64>)> {
    if let Ok(smaps) = std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")) {
        let mut rss = None;
        let mut pss = None;
        for line in smaps.lines() {
            if let Some(rest) = line.strip_prefix("Rss:") {
                rss = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok());
            } else if let Some(rest) = line.strip_prefix("Pss:") {
                pss = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok());
            }
        }
        if let Some(r) = rss {
            return Some((r, pss));
        }
    }
    if let Ok(status) = std::fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                if let Some(r) = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()) {
                    return Some((r, None));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct LinuxLiveTelemetry {
    webkit_ui: LiveStats,
    qt_ui: LiveStats,
    core: LiveStats,
    xray: LiveStats,
}

#[cfg(target_os = "linux")]
fn read_proc_ppid(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rparen = stat.rfind(')')?;
    let mut fields = stat[rparen + 1..].split_whitespace();
    let _state = fields.next()?;
    fields.next()?.parse::<u32>().ok()
}

#[cfg(target_os = "linux")]
fn collect_linux_telemetry() -> LinuxLiveTelemetry {
    static CACHE: OnceLock<Mutex<Option<(Instant, LinuxLiveTelemetry)>>> = OnceLock::new();
    const CACHE_TTL: Duration = Duration::from_secs(4);

    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some((at, ref snap)) = *guard {
            if at.elapsed() < CACHE_TTL {
                return snap.clone();
            }
        }
    }

    let mut telemetry = LinuxLiveTelemetry {
        webkit_ui: LiveStats::default(),
        qt_ui: LiveStats::default(),
        core: LiveStats::default(),
        xray: LiveStats::default(),
    };
    let my_pid = std::process::id();

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return telemetry;
    };

    let mut portmaster_pids = HashSet::new();
    let mut webkit_candidates = Vec::new();

    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };

        if pid == my_pid {
            if let Some((rss, pss)) = read_proc_rss_pss(pid) {
                telemetry.core.add(rss, pss);
            }
            continue;
        }

        // Read the short 16-byte comm name first so we skip ~98% of unrelated PIDs
        // without opening cmdline or smaps_rollup.
        let Ok(comm_raw) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        let comm = comm_raw.trim();

        if comm == "portmaster-qt" {
            if let Some((rss, pss)) = read_proc_rss_pss(pid) {
                telemetry.qt_ui.add(rss, pss);
            }
        } else if comm == "xray-rust" {
            if let Some((rss, pss)) = read_proc_rss_pss(pid) {
                telemetry.xray.add(rss, pss);
            }
        } else if comm == "portmaster" || comm == "portmaster-app" {
            let cmdline = std::fs::read(format!("/proc/{pid}/cmdline"))
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default();
            if !cmdline.contains("portmaster-core") && !cmdline.contains("portmaster-qt") {
                portmaster_pids.insert(pid);
                if let Some((rss, pss)) = read_proc_rss_pss(pid) {
                    telemetry.webkit_ui.add(rss, pss);
                }
            }
        } else if comm.starts_with("WebKit") {
            webkit_candidates.push(pid);
        }
    }

    for pid in webkit_candidates {
        if let Some(ppid) = read_proc_ppid(pid) {
            if portmaster_pids.contains(&ppid) {
                if let Some((rss, pss)) = read_proc_rss_pss(pid) {
                    telemetry.webkit_ui.add(rss, pss);
                }
            }
        }
    }

    if let Ok(mut guard) = cache.lock() {
        *guard = Some((Instant::now(), telemetry.clone()));
    }

    telemetry
}

pub fn build_flavors_report(
    config: &Arc<Config>,
    data_dir: &Path,
    bin_dir: &Path,
    port_ctrl: &ApiPortController,
    unix_socket_path: Option<&PathBuf>,
) -> UiFlavorsReport {
    let active_flavor = config.get_as_string(CFG_UI_FLAVOR, "angular")();
    let keep_api_port_open = config.get_as_bool(CFG_KEEP_API_PORT_OPEN, true)();

    let angular_installed = bin_dir.join("portmaster.zip").is_file()
        || data_dir.join("portmaster.zip").is_file()
        || data_dir.join("ui/modules/portmaster/index.html").is_file();

    let rust_dir = find_rust_ui_dir(data_dir, bin_dir);
    let rust_installed = rust_dir.join("index.html").is_file();

    let qt_bin = find_qt_binary(bin_dir, data_dir);
    let qt_installed = qt_bin.is_some();

    #[cfg(target_os = "linux")]
    let (webkit_live, qt_live, core_live, xray_live) = {
        let t = collect_linux_telemetry();
        (t.webkit_ui, t.qt_ui, t.core, t.xray)
    };
    #[cfg(not(target_os = "linux"))]
    let (webkit_live, qt_live, core_live, xray_live) = (
        LiveStats::default(),
        LiveStats::default(),
        LiveStats::default(),
        LiveStats::default(),
    );

    let webkit_runs_rust = active_flavor == "rust-wasm";
    let (angular_live_pss, angular_live_rss, angular_procs) = if !webkit_runs_rust && webkit_live.count > 0 {
        (webkit_live.effective_mb(), webkit_live.rss_mb(), webkit_live.count)
    } else {
        (None, None, 0)
    };
    let (rust_live_pss, rust_live_rss, rust_procs) = if webkit_runs_rust && webkit_live.count > 0 {
        (webkit_live.effective_mb(), webkit_live.rss_mb(), webkit_live.count)
    } else {
        (None, None, 0)
    };

    UiFlavorsReport {
        active_flavor: active_flavor.clone(),
        keep_api_port_open,
        api_port_listening: port_ctrl.is_listening(),
        api_port_address: port_ctrl.listen_addr().to_string(),
        unix_socket_path: unix_socket_path.map(|p| p.to_string_lossy().into_owned()),
        flavors: vec![
            UiFlavorInfo {
                id: "angular",
                name: "Angular Web UI",
                kind: "Web (Tauri WebKit/WebView2 or Browser)",
                installed: angular_installed,
                running: angular_procs > 0,
                active: active_flavor == "angular",
                live_ram_mb: angular_live_pss,
                live_rss_mb: angular_live_rss,
                benchmark_ram_mb: ANGULAR_BENCHMARK_TAURI_MB,
                browser_tab_ram_mb: Some(ANGULAR_BENCHMARK_TAB_MB),
                process_count: angular_procs,
                url: Some("/ui/modules/portmaster/"),
                binary_path: None,
            },
            UiFlavorInfo {
                id: "rust-wasm",
                name: "Rust WASM (Leptos)",
                kind: "WASM (WebAssembly in Browser or WebView)",
                installed: rust_installed,
                running: rust_procs > 0,
                active: active_flavor == "rust-wasm",
                live_ram_mb: rust_live_pss,
                live_rss_mb: rust_live_rss,
                benchmark_ram_mb: RUST_WASM_BENCHMARK_TAB_MB,
                browser_tab_ram_mb: Some(RUST_WASM_BENCHMARK_TAB_MB),
                process_count: rust_procs,
                url: Some("/rust/"),
                binary_path: Some(rust_dir.to_string_lossy().into_owned()),
            },
            UiFlavorInfo {
                id: "qt",
                name: "Qt 6 Native Desktop",
                kind: "Native C++/Qt (Zero WebKit / Zero Browser)",
                installed: qt_installed,
                running: qt_live.count > 0,
                active: active_flavor == "qt",
                live_ram_mb: qt_live.effective_mb(),
                live_rss_mb: qt_live.rss_mb(),
                benchmark_ram_mb: QT_BENCHMARK_MB,
                browser_tab_ram_mb: None,
                process_count: qt_live.count,
                url: None,
                binary_path: qt_bin.map(|p| p.to_string_lossy().into_owned()),
            },
        ],
        core_memory: ProcessMemoryInfo {
            live_rss_mb: core_live.rss_mb(),
            live_pss_mb: core_live.effective_mb(),
        },
        xray_memory: ProcessMemoryInfo {
            live_rss_mb: xray_live.rss_mb(),
            live_pss_mb: xray_live.effective_mb(),
        },
    }
}

pub fn register_ui_control_api(
    api: &Api,
    config: Arc<Config>,
    data_dir: PathBuf,
    bin_dir: PathBuf,
    port_ctrl: Arc<ApiPortController>,
    unix_socket_path: Option<PathBuf>,
) -> pm_api::Result<()> {
    let cfg_read = Arc::clone(&config);
    let data_read = data_dir.clone();
    let bin_read = bin_dir.clone();
    let ctrl_read = Arc::clone(&port_ctrl);
    let sock_read = unix_socket_path.clone();

    api.register_endpoint(
        Endpoint::new(
            "ui/flavors",
            "Available UI Flavors & RAM Metrics",
            "Lists Angular, Rust WASM, and Qt GUI options along with live and benchmark RAM usage.",
        )
        .with_read(PERMIT_USER)
        .with_struct(move |_| {
            let report = build_flavors_report(
                &cfg_read,
                &data_read,
                &bin_read,
                &ctrl_read,
                sock_read.as_ref(),
            );
            serde_json::to_value(report).map_err(pm_api::ApiError::Json)
        }),
    )?;

    let cfg_flavor = Arc::clone(&config);
    let data_flavor = data_dir.clone();
    let bin_flavor = bin_dir.clone();
    let ctrl_flavor = Arc::clone(&port_ctrl);
    let sock_flavor = unix_socket_path.clone();

    api.register_endpoint(
        Endpoint::new(
            "ui/flavor",
            "Set Active UI Flavor",
            "Switches core/uiFlavor between angular, rust-wasm, and qt.",
        )
        .with_write(PERMIT_ADMIN)
        .with_write_method("POST")
        .with_struct(move |request| {
            let payload: serde_json::Value = request.body_json()?;
            let flavor = payload
                .get("flavor")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if !matches!(flavor.as_str(), "angular" | "rust-wasm" | "qt") {
                return Err(pm_api::ApiError::WithStatus {
                    status: 400,
                    message: "flavor must be one of: angular, rust-wasm, qt".to_string(),
                });
            }
            cfg_flavor
                .set_config_option(CFG_UI_FLAVOR, serde_json::Value::String(flavor))
                .map_err(|e| pm_api::ApiError::Message(e.to_string()).with_status(400))?;
            let report = build_flavors_report(
                &cfg_flavor,
                &data_flavor,
                &bin_flavor,
                &ctrl_flavor,
                sock_flavor.as_ref(),
            );
            serde_json::to_value(report).map_err(pm_api::ApiError::Json)
        }),
    )?;

    let cfg_port = Arc::clone(&config);
    let ctrl_port = Arc::clone(&port_ctrl);
    api.register_endpoint(
        Endpoint::new(
            "core/api-port",
            "Control HTTP/WebSocket API Port State",
            "Toggles whether TCP 127.0.0.1:817 stays open continuously or opens on-demand.",
        )
        .with_write(PERMIT_ADMIN)
        .with_write_method("POST")
        .with_struct(move |request| {
            let payload: serde_json::Value = request.body_json()?;
            if let Some(keep_open) = payload.get("keepOpen").and_then(|v| v.as_bool()) {
                cfg_port
                    .set_config_option(CFG_KEEP_API_PORT_OPEN, serde_json::Value::Bool(keep_open))
                    .map_err(|e| pm_api::ApiError::Message(e.to_string()).with_status(400))?;
                ctrl_port.set_keep_open(keep_open);
            }
            if let Some(lease) = payload.get("lease").and_then(|v| v.as_str()) {
                match lease {
                    "open" => ctrl_port.set_web_lease(true),
                    "close" => ctrl_port.set_web_lease(false),
                    _ => {
                        return Err(pm_api::ApiError::WithStatus {
                            status: 400,
                            message: "lease must be 'open' or 'close'".to_string(),
                        });
                    }
                }
            }
            Ok(json!({
                "keepApiPortOpen": ctrl_port.keep_open(),
                "apiPortListening": ctrl_port.is_listening(),
                "address": ctrl_port.listen_addr().to_string(),
            }))
        }),
    )?;

    Ok(())
}
