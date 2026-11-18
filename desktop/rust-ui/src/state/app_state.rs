//! Global application state and reactive signals for Portmaster UI.

use leptos::prelude::*;
use pm_models::profile::AppProfile;
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

/// Global context holding UI signals and PortAPI interaction logic.
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub client: PortApiClient,
    pub active_view: ReadSignal<ViewTab>,
    pub set_active_view: WriteSignal<ViewTab>,
    pub route_by_default: ReadSignal<bool>,
    pub set_route_by_default: WriteSignal<bool>,
    pub xray_status: ReadSignal<XrayStatus>,
    pub set_xray_status: WriteSignal<XrayStatus>,
    pub xray_nodes: ReadSignal<Vec<XrayNode>>,
    pub set_xray_nodes: WriteSignal<Vec<XrayNode>>,
    pub profiles: ReadSignal<Vec<AppProfile>>,
    pub set_profiles: WriteSignal<Vec<AppProfile>>,
    pub is_busy: ReadSignal<bool>,
    pub set_busy: WriteSignal<bool>,
    pub filter_query: ReadSignal<String>,
    pub set_filter_query: WriteSignal<String>,
}

impl AppState {
    pub fn new(client: PortApiClient) -> Self {
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

        let (active_view, set_active_view) = signal(ViewTab::Apps);
        let (route_by_default, set_route_by_default) = signal(false);
        let (xray_status, set_xray_status) = signal(initial_status);
        let (xray_nodes, set_xray_nodes) = signal(initial_nodes);
        let (profiles, set_profiles) = signal(initial_profiles);
        let (is_busy, set_busy) = signal(false);
        let (filter_query, set_filter_query) = signal(String::new());

        let state = Self {
            client,
            active_view,
            set_active_view,
            route_by_default,
            set_route_by_default,
            xray_status,
            set_xray_status,
            xray_nodes,
            set_xray_nodes,
            profiles,
            set_profiles,
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

        // Subscribe to Xray status updates
        let _ = client.sub("xray/status", move |frame| {
            if let Ok(status) = frame.parse_data::<XrayStatus>() {
                set_xray_status.set(status);
            }
        });
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
                // Disconnect
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
                // Connect
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
