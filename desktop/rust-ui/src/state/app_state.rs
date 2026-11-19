//! Global application state and reactive signals for Portmaster UI.

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;
use pm_models::profile::AppProfile;
use pm_models::settings::{ExpertiseLevel, OptionType, Setting, SettingAnnotations};
use pm_models::xray::{ProcessState, XrayNode, XrayStatus};
use wasm_bindgen_futures::spawn_local;

use crate::client::PortApiClient;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTab {
    Apps,
    Vpn,
    Settings,
    Monitor,
}

impl Default for ViewTab {
    fn default() -> Self {
        Self::Apps
    }
}

/// Navigation pages for Portmaster UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Monitor,
    Apps,
    AppDetail(String),
    Xray,
    Settings,
    Support,
}

impl Default for Page {
    fn default() -> Self {
        Self::Apps
    }
}

/// A notification item for the notification drawer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub level: &'static str, // "info", "warning", "error"
    pub time: &'static str,
}

/// Global context holding UI signals and PortAPI interaction logic.
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub client: PortApiClient,
    pub current_page: ReadSignal<Page>,
    pub set_current_page: WriteSignal<Page>,
    pub active_view: ReadSignal<ViewTab>,
    pub set_active_view: WriteSignal<ViewTab>,
    pub selected_app: ReadSignal<Option<String>>,
    pub set_selected_app: WriteSignal<Option<String>>,
    pub is_side_dash_open: ReadSignal<bool>,
    pub set_side_dash_open: WriteSignal<bool>,
    pub route_by_default: ReadSignal<bool>,
    pub set_route_by_default: WriteSignal<bool>,
    pub expertise_level: ReadSignal<ExpertiseLevel>,
    pub set_expertise_level: WriteSignal<ExpertiseLevel>,
    pub security_level: ReadSignal<String>,
    pub set_security_level: WriteSignal<String>,
    pub is_paused: ReadSignal<bool>,
    pub set_paused: WriteSignal<bool>,
    pub pause_until: ReadSignal<Option<String>>,
    pub set_pause_until: WriteSignal<Option<String>>,
    pub notifications: ReadSignal<Vec<AppNotification>>,
    pub set_notifications: WriteSignal<Vec<AppNotification>>,

    pub xray_status: ReadSignal<XrayStatus>,
    pub set_xray_status: WriteSignal<XrayStatus>,
    pub xray_nodes: ReadSignal<Vec<XrayNode>>,
    pub set_xray_nodes: WriteSignal<Vec<XrayNode>>,
    pub profiles: ReadSignal<Vec<AppProfile>>,
    pub set_profiles: WriteSignal<Vec<AppProfile>>,
    pub settings: ReadSignal<Vec<Setting>>,
    pub set_settings: WriteSignal<Vec<Setting>>,
    pub connections: ReadSignal<Vec<ConnectionRecord>>,
    pub set_connections: WriteSignal<Vec<ConnectionRecord>>,
    pub pending_prompts: ReadSignal<usize>,
    pub set_pending_prompts: WriteSignal<usize>,

    pub is_busy: ReadSignal<bool>,
    pub set_busy: WriteSignal<bool>,
    pub filter_query: ReadSignal<String>,
    pub set_filter_query: WriteSignal<String>,
}

impl AppState {
    pub fn new(client: PortApiClient) -> Self {
        let (current_page, set_current_page) = signal(Page::Apps);
        let (active_view, set_active_view) = signal(ViewTab::Apps);
        let (selected_app, set_selected_app) = signal(None);
        let (is_side_dash_open, set_side_dash_open) = signal(false);
        let (route_by_default, set_route_by_default) = signal(false);
        let (expertise_level, set_expertise_level) = signal(ExpertiseLevel::User);
        let (security_level, set_security_level) = signal("Standard (Balanced)".to_string());
        let (is_paused, set_paused) = signal(false);
        let (pause_until, set_pause_until) = signal(None);

        let (notifications, set_notifications) = signal(Vec::<AppNotification>::new());
        let (xray_nodes, set_xray_nodes) = signal(Vec::<XrayNode>::new());
        let (xray_status, set_xray_status) = signal(XrayStatus::default());
        let (profiles, set_profiles) = signal(Vec::<AppProfile>::new());

        // Seed initial representative settings across subsystems
        let initial_settings = vec![
            Setting {
                key: "filter/defaultAction".to_string(),
                name: "Default Network Action".to_string(),
                description: "The default firewall action taken when no specific rule matches a connection.".to_string(),
                help: "Choose whether unclassified connections should be allowed, blocked or prompt the user.".to_string(),
                opt_type: OptionType::String,
                value: Some(serde_json::json!("permit")),
                default_value: serde_json::json!("ask"),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Rules & Filtering".to_string()),
                    display_hint: Some("one-of".to_string()),
                    ..Default::default()
                },
                possible_values: vec![
                    pm_models::settings::PossibleValue {
                        name: "Permit".to_string(),
                        description: "Allow connection without prompting".to_string(),
                        value: serde_json::json!("permit"),
                    },
                    pm_models::settings::PossibleValue {
                        name: "Block".to_string(),
                        description: "Drop connection immediately".to_string(),
                        value: serde_json::json!("block"),
                    },
                    pm_models::settings::PossibleValue {
                        name: "Ask".to_string(),
                        description: "Prompt user for interactive decision".to_string(),
                        value: serde_json::json!("ask"),
                    },
                ],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "filter/endpoints".to_string(),
                name: "Global Endpoint Rules".to_string(),
                description: "Rules for outbound network endpoints (IP addresses and domain names).".to_string(),
                help: "Specify rules like 'example.com block' or '1.1.1.1 permit'.".to_string(),
                opt_type: OptionType::StringArray,
                value: Some(serde_json::json!([
                    "analytics.google.com block",
                    "telemetry.microsoft.com block",
                    "1.1.1.1 permit"
                ])),
                default_value: serde_json::json!([]),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Rules & Filtering".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "dns/servers".to_string(),
                name: "DNS-over-TLS Upstream Servers".to_string(),
                description: "List of encrypted DNS servers used for name resolution.".to_string(),
                help: "Enter TLS DNS endpoints, e.g. 'tls://1.1.1.1' or 'tls://dns.quad9.net'.".to_string(),
                opt_type: OptionType::StringArray,
                value: None,
                default_value: serde_json::json!([
                    "tls://1.1.1.1",
                    "tls://9.9.9.9",
                    "tls://dns.adguard-dns.com"
                ]),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Resolver".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "dns/blockMalicious".to_string(),
                name: "Block Malicious Domains".to_string(),
                description: "Automatically drop DNS queries directed towards confirmed malware and phishing domains.".to_string(),
                help: "Protects all applications from known malware command-and-control servers.".to_string(),
                opt_type: OptionType::Bool,
                value: Some(serde_json::json!(true)),
                default_value: serde_json::json!(true),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Resolver".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "xray/enable".to_string(),
                name: "Enable Xray VPN Subsystem".to_string(),
                description: "Activates the integrated Xray proxy and VPN background tunnel engine.".to_string(),
                help: "Allows routing applications through high-speed encrypted VLESS/Trojan proxies.".to_string(),
                opt_type: OptionType::Bool,
                value: Some(serde_json::json!(true)),
                default_value: serde_json::json!(true),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("General".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "xray/inboundPort".to_string(),
                name: "Xray Inbound Local Port".to_string(),
                description: "Local TCP port on 127.0.0.1 where the Xray SOCKS5/HTTP inbound proxy listens.".to_string(),
                help: "Default is 10808.".to_string(),
                opt_type: OptionType::Int,
                value: None,
                default_value: serde_json::json!(10808),
                expertise_level: 1,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Routing".to_string()),
                    unit: Some("port".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: true,
                meta: None,
            },
            Setting {
                key: "splittun/usagePolicy".to_string(),
                name: "Default Tunnel Policy".to_string(),
                description: "Global fallback policy for split tunneling.".to_string(),
                help: "Direct routes bypass the VPN, whereas VPN routes all application traffic through the active tunnel.".to_string(),
                opt_type: OptionType::String,
                value: Some(serde_json::json!("direct")),
                default_value: serde_json::json!("direct"),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("General".to_string()),
                    display_hint: Some("one-of".to_string()),
                    ..Default::default()
                },
                possible_values: vec![
                    pm_models::settings::PossibleValue {
                        name: "Direct".to_string(),
                        description: "Bypass VPN and route via local ISP connection".to_string(),
                        value: serde_json::json!("direct"),
                    },
                    pm_models::settings::PossibleValue {
                        name: "VPN / Proxy".to_string(),
                        description: "Route via active VPN gateway".to_string(),
                        value: serde_json::json!("vpn"),
                    },
                ],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "history/retentionDays".to_string(),
                name: "Network History Retention".to_string(),
                description: "Number of days connection records are stored in the local SQLite database.".to_string(),
                help: "Set to 0 to disable persistent connection logging.".to_string(),
                opt_type: OptionType::Int,
                value: Some(serde_json::json!(7)),
                default_value: serde_json::json!(7),
                expertise_level: 0,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("Database".to_string()),
                    unit: Some("days".to_string()),
                    ..Default::default()
                },
                possible_values: vec![],
                requires_restart: false,
                meta: None,
            },
            Setting {
                key: "core/releaseLevel".to_string(),
                name: "Feature Release Channel".to_string(),
                description: "Select whether experimental or beta features and settings should be exposed.".to_string(),
                help: "Stable, Beta or Experimental.".to_string(),
                opt_type: OptionType::String,
                value: Some(serde_json::json!("stable")),
                default_value: serde_json::json!("stable"),
                expertise_level: 1,
                release_level: 0,
                annotations: SettingAnnotations {
                    category: Some("General".to_string()),
                    display_hint: Some("one-of".to_string()),
                    ..Default::default()
                },
                possible_values: vec![
                    pm_models::settings::PossibleValue {
                        name: "Stable".to_string(),
                        description: "Recommended for production systems".to_string(),
                        value: serde_json::json!("stable"),
                    },
                    pm_models::settings::PossibleValue {
                        name: "Beta".to_string(),
                        description: "Early access to upcoming features".to_string(),
                        value: serde_json::json!("beta"),
                    },
                    pm_models::settings::PossibleValue {
                        name: "Experimental".to_string(),
                        description: "Cutting-edge features in testing".to_string(),
                        value: serde_json::json!("experimental"),
                    },
                ],
                requires_restart: false,
                meta: None,
            },
        ];

        let (connections, set_connections) = signal(Vec::<ConnectionRecord>::new());
        let (pending_prompts, set_pending_prompts) = signal(0);
        let (settings, set_settings) = signal(initial_settings);
        let (is_busy, set_busy) = signal(false);
        let (filter_query, set_filter_query) = signal(String::new());

        let state = Self {
            client,
            current_page,
            set_current_page,
            active_view,
            set_active_view,
            selected_app,
            set_selected_app,
            is_side_dash_open,
            set_side_dash_open,
            route_by_default,
            set_route_by_default,
            expertise_level,
            set_expertise_level,
            security_level,
            set_security_level,
            is_paused,
            set_paused,
            pause_until,
            set_pause_until,
            notifications,
            set_notifications,

            xray_status,
            set_xray_status,
            xray_nodes,
            set_xray_nodes,
            profiles,
            set_profiles,
            settings,
            set_settings,
            connections,
            set_connections,
            pending_prompts,
            set_pending_prompts,

            is_busy,
            set_busy,
            filter_query,
            set_filter_query,
        };

        // Subscribe to live database streams
        state.init_subscriptions();

        state
    }

    /// Sets up real-time queries and subscriptions to Portmaster database.
    fn init_subscriptions(&self) {
        let client = self.client.clone();
        let set_profiles = self.set_profiles;
        let set_xray_status = self.set_xray_status;
        let set_xray_nodes = self.set_xray_nodes;
        let set_settings = self.set_settings;
        let set_connections = self.set_connections;

        let base_url = if let Some(window) = web_sys::window() {
            let loc = window.location();
            if let Ok(host) = loc.host() {
                if !host.is_empty() && (host.ends_with(":817") || host == "127.0.0.1:817" || host == "localhost:817") {
                    let proto = loc.protocol().unwrap_or_else(|_| "http:".to_string());
                    format!("{}//{}", proto, host)
                } else {
                    "http://127.0.0.1:817".to_string()
                }
            } else {
                "http://127.0.0.1:817".to_string()
            }
        } else {
            "http://127.0.0.1:817".to_string()
        };

        // 1. Fetch real Xray Nodes from /api/v1/xray/nodes
        let nodes_url = format!("{}/api/v1/xray/nodes", base_url);
        spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get(&nodes_url).send().await {
                if let Ok(nodes) = resp.json::<Vec<XrayNode>>().await {
                    set_xray_nodes.set(nodes);
                }
            }
        });

        // 2. Fetch real Xray Status from /api/v1/xray/status
        let status_url = format!("{}/api/v1/xray/status", base_url);
        spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get(&status_url).send().await {
                if let Ok(status) = resp.json::<XrayStatus>().await {
                    set_xray_status.set(status);
                }
            }
        });

        // 3. Fetch real initial connections from /api/v1/netquery/query
        let nq_url = format!("{}/api/v1/netquery/query", base_url);
        let nq_set_conns = set_connections;
        let nq_set_profiles = set_profiles;
        spawn_local(async move {
            let query_payload = serde_json::json!({
                "databases": ["main"],
                "select": [
                    "id", "profile", "profile_name", "domain", "remote_ip", "remote_port",
                    "ip_protocol", "direction", "scope", "country", "verdict",
                    "bytes_sent", "bytes_received", "active"
                ]
            });
            if let Ok(resp) = gloo_net::http::Request::post(&nq_url)
                .header("Content-Type", "application/json")
                .body(query_payload.to_string())
                .unwrap()
                .send()
                .await
            {
                #[derive(serde::Deserialize)]
                struct NqResponse {
                    results: Option<Vec<ConnectionRecord>>,
                }
                if let Ok(nq) = resp.json::<NqResponse>().await {
                    if let Some(mut conns) = nq.results {
                        // Reverse so newest connections are on top
                        conns.reverse();
                        nq_set_conns.set(conns.clone());

                        // Update or populate profiles bandwidth & active counts from real connections
                        nq_set_profiles.update(|profs| {
                            for c in &conns {
                                let prof_id = if let Some((_, id)) = c.profile_id.split_once('/') {
                                    id.to_string()
                                } else {
                                    c.profile_id.clone()
                                };
                                let target = if let Some(p) = profs.iter_mut().find(|p| p.id == prof_id || p.id == c.profile_id || p.name.eq_ignore_ascii_case(&c.profile_name)) {
                                    p
                                } else {
                                    let mut new_p = AppProfile::new(prof_id, &c.profile_name);
                                    if let Some((src, _)) = c.profile_id.split_once('/') {
                                        new_p.source = src.to_string();
                                    }
                                    profs.push(new_p);
                                    profs.last_mut().unwrap()
                                };

                                let stats = target.stats.get_or_insert_with(Default::default);
                                if c.active {
                                    stats.connections_count = stats.connections_count.saturating_add(1);
                                }
                                if c.is_blocked() {
                                    stats.blocked_count = stats.blocked_count.saturating_add(1);
                                }
                                stats.bytes_sent = stats.bytes_sent.saturating_add(c.bytes_sent);
                                stats.bytes_received = stats.bytes_received.saturating_add(c.bytes_received);
                            }
                        });
                    }
                }
            }
        });

        // 4. Subscribe to profiles updates
        client.qsub("core:profiles/", move |frame| {
            if let Ok(profile) = frame.parse_data::<AppProfile>() {
                set_profiles.update(|list| {
                    if let Some(pos) = list.iter().position(|p| p.id == profile.id || p.name.eq_ignore_ascii_case(&profile.name)) {
                        let mut updated = profile;
                        if updated.stats.is_none() && list[pos].stats.is_some() {
                            updated.stats = list[pos].stats;
                        }
                        list[pos] = updated;
                    } else {
                        list.push(profile);
                    }
                });
            }
        });

        // 5. Subscribe to settings updates
        client.qsub("config:options/", move |frame| {
            if let Ok(setting) = frame.parse_data::<Setting>() {
                set_settings.update(|list| {
                    if let Some(pos) = list.iter().position(|s| s.key == setting.key) {
                        list[pos] = setting;
                    } else {
                        list.push(setting);
                    }
                });
            }
        });

        // 6. Subscribe to Xray status updates
        let _ = client.sub("xray/status", move |frame| {
            if let Ok(status) = frame.parse_data::<XrayStatus>() {
                set_xray_status.set(status);
            }
        });

        // 7. Subscribe to real-time connection events via netquery/data/
        let live_set_conns = set_connections;
        let live_set_profiles = set_profiles;
        client.qsub("netquery/data/", move |frame| {
            if let Ok(conn) = frame.parse_data::<ConnectionRecord>() {
                let c_clone = conn.clone();
                let mut prev_sent = 0u64;
                let mut prev_recv = 0u64;
                let mut is_new = false;

                live_set_conns.update(|list| {
                    if let Some(pos) = list.iter().position(|c| c.id == conn.id) {
                        prev_sent = list[pos].bytes_sent;
                        prev_recv = list[pos].bytes_received;
                        list[pos] = conn;
                    } else {
                        is_new = true;
                        list.insert(0, conn);
                        if list.len() > 500 {
                            list.truncate(500);
                        }
                    }
                });

                live_set_profiles.update(|profs| {
                    let prof_id = if let Some((_, id)) = c_clone.profile_id.split_once('/') {
                        id.to_string()
                    } else {
                        c_clone.profile_id.clone()
                    };
                    let p = if let Some(p) = profs.iter_mut().find(|p| p.id == prof_id || p.id == c_clone.profile_id || p.name.eq_ignore_ascii_case(&c_clone.profile_name)) {
                        p
                    } else {
                        let mut new_p = AppProfile::new(prof_id, &c_clone.profile_name);
                        if let Some((src, _)) = c_clone.profile_id.split_once('/') {
                            new_p.source = src.to_string();
                        }
                        profs.push(new_p);
                        profs.last_mut().unwrap()
                    };

                    let stats = p.stats.get_or_insert_with(Default::default);
                    if is_new {
                        if c_clone.active {
                            stats.connections_count = stats.connections_count.saturating_add(1);
                        }
                        if c_clone.is_blocked() {
                            stats.blocked_count = stats.blocked_count.saturating_add(1);
                        }
                        stats.bytes_sent = stats.bytes_sent.saturating_add(c_clone.bytes_sent);
                        stats.bytes_received = stats.bytes_received.saturating_add(c_clone.bytes_received);
                    } else {
                        let delta_sent = c_clone.bytes_sent.saturating_sub(prev_sent);
                        let delta_recv = c_clone.bytes_received.saturating_sub(prev_recv);
                        stats.bytes_sent = stats.bytes_sent.saturating_add(delta_sent);
                        stats.bytes_received = stats.bytes_received.saturating_add(delta_recv);
                    }
                });
            }
        });
    }

    /// Navigates to a specific UI page and keeps view signals in sync.
    pub fn navigate(&self, page: Page) {
        match &page {
            Page::Apps => {
                self.set_selected_app.set(None);
                self.set_active_view.set(ViewTab::Apps);
            }
            Page::AppDetail(id) => {
                self.set_selected_app.set(Some(id.clone()));
                self.set_active_view.set(ViewTab::Apps);
            }
            Page::Xray => {
                self.set_active_view.set(ViewTab::Vpn);
            }
            Page::Settings => {
                self.set_active_view.set(ViewTab::Settings);
            }
            Page::Monitor => {
                self.set_active_view.set(ViewTab::Monitor);
            }
            Page::Support => {}
        }
        self.set_current_page.set(page);
    }

    /// Selects an application to open in detailed AppView.
    pub fn select_app(&self, app_id: Option<String>) {
        if let Some(id) = &app_id {
            self.navigate(Page::AppDetail(id.clone()));
        } else {
            self.navigate(Page::Apps);
        }
    }

    /// Toggles SideDash drawer.
    pub fn toggle_side_dash(&self) {
        self.set_side_dash_open.update(|v| *v = !*v);
    }

    /// Toggles "Route Through VPN (Xray) by Default" global setting.
    pub fn toggle_route_by_default(&self) {
        let current = self.route_by_default.get_untracked();
        let new_val = !current;
        self.set_route_by_default.set(new_val);

        let client = self.client.clone();
        spawn_local(async move {
            let _ = client
                .put(
                    "update",
                    "config:spn/use",
                    serde_json::json!({
                        "Key": "spn/use",
                        "Value": new_val
                    }),
                )
                .await;
        });
    }

    /// Pauses filtering for the given duration in seconds.
    pub fn pause_protection(&self, seconds: u64) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let client = self.client.clone();

        spawn_local(async move {
            let mins = seconds / 60;
            let time_str = format!("Paused for {}m", mins);
            set_paused.set(true);
            set_pause_until.set(Some(time_str));

            let _ = client
                .put(
                    "update",
                    "filter:pause",
                    serde_json::json!({ "duration": seconds }),
                )
                .await;
        });
    }

    /// Resumes protection.
    pub fn resume_protection(&self) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let client = self.client.clone();

        spawn_local(async move {
            set_paused.set(false);
            set_pause_until.set(None);

            let _ = client
                .put(
                    "update",
                    "filter:resume",
                    serde_json::json!({ "resume": true }),
                )
                .await;
        });
    }

    /// Saves a setting value to core.
    pub fn save_setting(&self, key: impl Into<String>, value: Option<serde_json::Value>) {
        let key = key.into();
        let set_settings = self.set_settings;
        let client = self.client.clone();

        set_settings.update(|list| {
            if let Some(s) = list.iter_mut().find(|item| item.key == key) {
                s.value = value.clone();
                let db_key = s.database_key();
                let effective_val = s.effective_value().clone();

                spawn_local(async move {
                    let _ = client.put("update", &db_key, effective_val).await;
                });
            }
        });
    }

    /// Updates a setting with a new value.
    pub fn update_setting(&self, key: impl Into<String>, value: serde_json::Value) {
        self.save_setting(key, Some(value));
    }

    /// Resets a setting to default.
    pub fn reset_setting(&self, key: impl Into<String>) {
        self.save_setting(key, None);
    }

    /// Pauses protection for specified duration in minutes.
    pub fn pause(&self, minutes: u32, vpn_only: bool) {
        if vpn_only {
            self.set_xray_status.update(|s| s.connected = false);
            self.set_paused.set(true);
            self.set_pause_until.set(Some(format!("VPN Paused for {}m", minutes)));
        } else {
            self.pause_protection(minutes as u64 * 60);
        }
    }

    /// Resumes all protection.
    pub fn resume(&self) {
        self.resume_protection();
    }

    /// Toggles VPN on/off.
    pub fn toggle_vpn(&self) {
        let status = self.xray_status.get_untracked();
        let set_status = self.set_xray_status;
        let client = self.client.clone();
        let set_busy = self.set_busy;

        set_busy.set(true);

        spawn_local(async move {
            if status.connected {
                let _ = client
                    .put(
                        "update",
                        "xray:status",
                        serde_json::json!({ "connected": false, "state": "stopped" }),
                    )
                    .await;

                set_status.update(|s| {
                    s.connected = false;
                    s.state = ProcessState::Stopped;
                });
            } else {
                let _ = client
                    .put(
                        "update",
                        "xray:status",
                        serde_json::json!({ "connected": true, "state": "running" }),
                    )
                    .await;

                set_status.update(|s| {
                    s.connected = true;
                    s.state = ProcessState::Running;
                });
            }
            set_busy.set(false);
        });
    }

    /// Selects an active VPN node.
    pub fn select_node(&self, node_id: String) {
        let nodes = self.xray_nodes.get_untracked();
        let Some(target) = nodes.into_iter().find(|n| n.id == node_id) else {
            return;
        };

        let set_status = self.set_xray_status;
        let client = self.client.clone();

        spawn_local(async move {
            let _ = client
                .put(
                    "update",
                    "xray:connect",
                    serde_json::json!({ "nodeId": target.id }),
                )
                .await;

            set_status.update(|s| {
                s.node_id = Some(target.id.clone());
                s.node_name = Some(target.name.clone());
                s.country = Some(target.country_code.clone());
                s.exit_country = Some(target.country_code.clone());
                s.connected = true;
                s.state = ProcessState::Running;
            });
        });
    }

    /// Toggles whether a specific application routes its traffic via VPN.
    pub fn toggle_app_vpn(&self, app_id: String) {
        let set_profiles = self.set_profiles;
        let client = self.client.clone();

        set_profiles.update(|list| {
            if let Some(p) = list.iter_mut().find(|item| item.id == app_id) {
                let current_vpn = p.route_via_vpn();
                let next_vpn = !current_vpn;
                p.set_route_via_vpn(next_vpn);

                let key = p.database_key();
                let profile_val = serde_json::to_value(&*p).ok();

                spawn_local(async move {
                    if let Some(val) = profile_val {
                        let _ = client.put("update", &key, val).await;
                    }
                });
            }
        });
    }

    /// Changes an application default filter action ("permit", "block", "ask").
    pub fn set_app_action(&self, app_id: String, action: String) {
        let set_profiles = self.set_profiles;
        let client = self.client.clone();

        set_profiles.update(|list| {
            if let Some(p) = list.iter_mut().find(|item| item.id == app_id) {
                p.set_default_action(&action);

                let key = p.database_key();
                let profile_val = serde_json::to_value(&*p).ok();

                spawn_local(async move {
                    if let Some(val) = profile_val {
                        let _ = client.put("update", &key, val).await;
                    }
                });
            }
        });
    }

    /// Explicitly permits a connection and updates its record.
    pub fn allow_connection(&self, conn_id: &str) {
        let conn_id = conn_id.to_string();
        self.set_connections.update(|conns| {
            if let Some(c) = conns.iter_mut().find(|c| c.id == conn_id) {
                c.verdict = "permit".to_string();
                c.verdict_reason = "Manual Rule: Explicitly Allowed by User".to_string();
            }
        });
    }

    /// Explicitly blocks a connection and updates its record.
    pub fn block_connection(&self, conn_id: &str) {
        let conn_id = conn_id.to_string();
        self.set_connections.update(|conns| {
            if let Some(c) = conns.iter_mut().find(|c| c.id == conn_id) {
                c.verdict = "block".to_string();
                c.verdict_reason = "Manual Rule: Explicitly Blocked by User".to_string();
            }
        });
    }

    /// Clears network activity history.
    pub fn clear_history(&self) {
        self.set_connections.set(Vec::new());
    }
}
