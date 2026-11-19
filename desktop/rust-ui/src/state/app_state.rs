//! Global application state and reactive signals for Portmaster UI.

use leptos::prelude::*;
use pm_models::profile::AppProfile;
use pm_models::settings::{ExpertiseLevel, OptionType, Setting, SettingAnnotations};
use pm_models::xray::{ProcessState, TrafficStatus, XrayNode, XrayStatus};
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

        let initial_notifications = vec![
            AppNotification {
                id: "notif-1".to_string(),
                title: "Core Service Online".to_string(),
                message: "Portmaster v1.0 engine started and listening on 127.0.0.1:817".to_string(),
                level: "info",
                time: "Just now",
            },
            AppNotification {
                id: "notif-2".to_string(),
                title: "Tracker Blocked".to_string(),
                message: "Blocked 14 outbound connections to telemetry endpoints".to_string(),
                level: "warning",
                time: "2m ago",
            },
        ];
        let (notifications, set_notifications) = signal(initial_notifications);

        // Initial mock/seed nodes for instant interactivity
        let initial_nodes = vec![
            XrayNode {
                id: "node-nl-01".to_string(),
                subscription_id: "sub-main".to_string(),
                name: "Amsterdam Fast #1".to_string(),
                country_code: "NL".to_string(),
                country_name: "Netherlands".to_string(),
                protocol: "vless".to_string(),
                address: "amsterdam.fastproxy.net".to_string(),
                port: 443,
                ping_ms: Some(28),
            },
            XrayNode {
                id: "node-de-01".to_string(),
                subscription_id: "sub-main".to_string(),
                name: "Frankfurt Core #2".to_string(),
                country_code: "DE".to_string(),
                country_name: "Germany".to_string(),
                protocol: "vless".to_string(),
                address: "frankfurt.coreproxy.net".to_string(),
                port: 443,
                ping_ms: Some(34),
            },
            XrayNode {
                id: "node-ch-01".to_string(),
                subscription_id: "sub-main".to_string(),
                name: "Zurich Privacy Shield".to_string(),
                country_code: "CH".to_string(),
                country_name: "Switzerland".to_string(),
                protocol: "trojan".to_string(),
                address: "zurich.safeproxy.ch".to_string(),
                port: 443,
                ping_ms: Some(42),
            },
            XrayNode {
                id: "node-us-01".to_string(),
                subscription_id: "sub-main".to_string(),
                name: "New York East #1".to_string(),
                country_code: "US".to_string(),
                country_name: "United States".to_string(),
                protocol: "vmess".to_string(),
                address: "ny.eastproxy.io".to_string(),
                port: 8443,
                ping_ms: Some(95),
            },
            XrayNode {
                id: "node-jp-01".to_string(),
                subscription_id: "sub-main".to_string(),
                name: "Tokyo Ultra Fast".to_string(),
                country_code: "JP".to_string(),
                country_name: "Japan".to_string(),
                protocol: "hysteria".to_string(),
                address: "tokyo.fastproxy.jp".to_string(),
                port: 443,
                ping_ms: Some(180),
            },
        ];

        let initial_status = XrayStatus {
            enabled: true,
            connected: true,
            state: ProcessState::Running,
            node_id: Some("node-nl-01".to_string()),
            node_name: Some("Amsterdam Fast #1".to_string()),
            country: Some("NL".to_string()),
            traffic: Some(TrafficStatus {
                uplink: 1024 * 1024 * 42,
                downlink: 1024 * 1024 * 238,
            }),
            exit_ip: Some("194.36.191.12".to_string()),
            exit_country: Some("NL".to_string()),
            ..Default::default()
        };

        // Seed initial applications
        let mut initial_profiles = Vec::new();

        let mut tg = AppProfile::new("telegram", "Telegram Desktop");
        tg.presentation_path = "/usr/bin/telegram-desktop".to_string();
        tg.set_route_via_vpn(true);
        tg.set_default_action("permit");
        initial_profiles.push(tg);

        let mut ff = AppProfile::new("firefox", "Mozilla Firefox");
        ff.presentation_path = "/usr/bin/firefox".to_string();
        ff.set_route_via_vpn(true);
        ff.set_default_action("permit");
        initial_profiles.push(ff);

        let mut discord = AppProfile::new("discord", "Discord");
        discord.presentation_path = "/usr/bin/discord".to_string();
        discord.set_route_via_vpn(false);
        discord.set_default_action("permit");
        initial_profiles.push(discord);

        let mut steam = AppProfile::new("steam", "Steam");
        steam.presentation_path = "/usr/bin/steam".to_string();
        steam.set_route_via_vpn(false);
        steam.set_default_action("permit");
        initial_profiles.push(steam);

        let mut term = AppProfile::new("terminal", "Alacritty Terminal");
        term.presentation_path = "/usr/bin/alacritty".to_string();
        term.set_route_via_vpn(false);
        term.set_default_action("ask");
        initial_profiles.push(term);

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

        let (xray_status, set_xray_status) = signal(initial_status);
        let (xray_nodes, set_xray_nodes) = signal(initial_nodes);
        let (profiles, set_profiles) = signal(initial_profiles);
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
        let set_settings = self.set_settings;

        // Subscribe to profiles updates
        client.qsub("core:profiles/", move |frame| {
            if let Ok(profile) = frame.parse_data::<AppProfile>() {
                set_profiles.update(|list| {
                    if let Some(pos) = list.iter().position(|p| p.id == profile.id) {
                        list[pos] = profile;
                    } else {
                        list.push(profile);
                    }
                });
            }
        });

        // Subscribe to settings updates
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

        // Subscribe to Xray status updates
        let _ = client.sub("xray/status", move |frame| {
            if let Ok(status) = frame.parse_data::<XrayStatus>() {
                set_xray_status.set(status);
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
    pub fn save_setting(&self, key: String, value: Option<serde_json::Value>) {
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

    /// Resets a setting to default.
    pub fn reset_setting(&self, key: String) {
        self.save_setting(key, None);
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
                let config_val = p.config.clone();

                spawn_local(async move {
                    if let Some(cfg) = config_val {
                        let _ = client.put("update", &key, cfg).await;
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
                let config_val = p.config.clone();

                spawn_local(async move {
                    if let Some(cfg) = config_val {
                        let _ = client.put("update", &key, cfg).await;
                    }
                });
            }
        });
    }
}
