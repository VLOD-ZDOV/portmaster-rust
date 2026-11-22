//! Global application state and reactive signals for Portmaster UI.

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;
use pm_models::profile::AppProfile;
use pm_models::settings::{ExpertiseLevel, Setting};
use pm_models::xray::{ProcessState, XrayNode, XrayStatus};
use wasm_bindgen_futures::spawn_local;

use crate::client::PortApiClient;

/// Helper to determine the API base URL from window location or default to 127.0.0.1:817.
pub fn api_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let loc = window.location();
        if let Ok(host) = loc.host() {
            if !host.is_empty() && (host.ends_with(":817") || host == "127.0.0.1:817" || host == "localhost:817") {
                let proto = loc.protocol().unwrap_or_else(|_| "http:".to_string());
                return format!("{}//{}", proto, host);
            }
        }
    }
    "http://127.0.0.1:817".to_string()
}

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

        let (connections, set_connections) = signal(Vec::<ConnectionRecord>::new());
        let (pending_prompts, set_pending_prompts) = signal(0);
        let (settings, set_settings) = signal(Vec::<Setting>::new());
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

        let base_url = api_base_url();

        // 1. Fetch real Xray Nodes from /api/v1/xray/nodes
        let nodes_url = format!("{}/api/v1/xray/nodes", base_url);
        spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get(&nodes_url).send().await {
                if let Ok(nodes) = resp.json::<Vec<XrayNode>>().await {
                    set_xray_nodes.set(nodes);
                }
            }
        });

        // 2. Fetch real Xray Status and poll it periodically
        let status_url = format!("{}/api/v1/xray/status", base_url);
        spawn_local(async move {
            loop {
                if let Ok(resp) = gloo_net::http::Request::get(&status_url).send().await {
                    if let Ok(status) = resp.json::<XrayStatus>().await {
                        set_xray_status.set(status);
                    }
                }
                crate::client::ws::sleep_ms(3000).await;
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
            if frame.msg_type == "del" {
                let parsed_id = frame.parse_data::<AppProfile>().map(|p| p.id).ok();
                let key_str = frame.key.as_deref().unwrap_or("");
                let key_id = key_str.rsplit('/').next().unwrap_or(key_str);
                set_profiles.update(|list| {
                    list.retain(|p| {
                        if let Some(id) = &parsed_id {
                            if p.id == *id {
                                return false;
                            }
                        }
                        if !key_str.is_empty() {
                            if p.id == key_str || p.id == key_id || key_str.ends_with(&format!("/{}", p.id)) {
                                return false;
                            }
                        }
                        true
                    });
                });
            } else if let Ok(profile) = frame.parse_data::<AppProfile>() {
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
        client.qsub("config:", move |frame| {
            if frame.msg_type == "del" {
                if let Some(key) = &frame.key {
                    let setting_key = key.strip_prefix("config:").unwrap_or(key);
                    set_settings.update(|list| {
                        list.retain(|s| s.key != setting_key);
                    });
                }
            } else if let Ok(setting) = frame.parse_data::<Setting>() {
                set_settings.update(|list| {
                    if let Some(pos) = list.iter().position(|s| s.key == setting.key) {
                        list[pos] = setting;
                    } else {
                        list.push(setting);
                    }
                });
            }
        });

        // 6. Subscribe to notifications updates
        let notif_set = self.set_notifications;
        let notif_prompts = self.set_pending_prompts;
        client.qsub("notifications:all/", move |frame| {
            if frame.msg_type == "del" {
                if let Some(key) = &frame.key {
                    let id_to_remove = key.strip_prefix("notifications:all/").unwrap_or(key);
                    notif_set.update(|list: &mut Vec<AppNotification>| {
                        list.retain(|n| n.id != id_to_remove);
                    });
                }
            } else if let Ok(val) = frame.parse_data::<serde_json::Value>() {
                let id = val.get("EventID").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = val.get("Title").and_then(|v| v.as_str()).unwrap_or("Notification").to_string();
                let message = val.get("Message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notif_type = val.get("Type").and_then(|v| v.as_str()).unwrap_or("info");
                let level = match notif_type {
                    "prompt" => {
                        notif_prompts.update(|cnt| *cnt = cnt.saturating_add(1));
                        "warning"
                    }
                    "error" => "error",
                    "warning" => "warning",
                    _ => "info",
                };
                if !id.is_empty() {
                    notif_set.update(|list: &mut Vec<AppNotification>| {
                        if let Some(pos) = list.iter().position(|n| n.id == id) {
                            list[pos] = AppNotification {
                                id,
                                title,
                                message,
                                level,
                                time: "Now",
                            };
                        } else {
                            list.push(AppNotification {
                                id,
                                title,
                                message,
                                level,
                                time: "Now",
                            });
                        }
                    });
                }
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
        let set_route_by_default = self.set_route_by_default;

        let client = self.client.clone();
        spawn_local(async move {
            if client
                .put(
                    "update",
                    "config:spn/use",
                    serde_json::json!({
                        "Key": "spn/use",
                        "Value": new_val
                    }),
                )
                .await
                .is_ok()
            {
                set_route_by_default.set(new_val);
            }
        });
    }

    /// Pauses filtering for the given duration in seconds.
    #[allow(dead_code)]
    pub fn pause_protection(&self, seconds: u64) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let url = format!("{}/api/v1/control/pause", api_base_url());
        let payload = serde_json::json!({
            "duration": seconds,
            "onlySPN": false,
        });

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        let mins = seconds / 60;
                        let time_str = format!("Paused for {}m", mins);
                        set_paused.set(true);
                        set_pause_until.set(Some(time_str));
                    }
                }
            }
        });
    }

    /// Resumes protection.
    pub fn resume_protection(&self) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let url = format!("{}/api/v1/control/resume", api_base_url());
        let payload = serde_json::json!({});

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_paused.set(false);
                        set_pause_until.set(None);
                    }
                }
            }
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

                spawn_local(async move {
                    if let Some(val) = value {
                        let _ = client
                            .put(
                                "update",
                                &db_key,
                                serde_json::json!({
                                    "Key": key,
                                    "Value": val,
                                }),
                            )
                            .await;
                    } else {
                        let _ = client.delete(&db_key).await;
                    }
                });
            }
        });
    }

    /// Updates a setting with a new value.
    pub fn update_setting(&self, key: impl Into<String>, value: serde_json::Value) {
        self.save_setting(key, Some(value));
    }

    /// Resets a setting to default by deleting its override.
    pub fn reset_setting(&self, key: impl Into<String>) {
        let key = key.into();
        let set_settings = self.set_settings;
        let client = self.client.clone();

        set_settings.update(|list| {
            if let Some(s) = list.iter_mut().find(|item| item.key == key) {
                s.value = None;
                let db_key = s.database_key();

                spawn_local(async move {
                    let _ = client.delete(&db_key).await;
                });
            }
        });
    }

    /// Pauses protection for specified duration in minutes.
    pub fn pause(&self, minutes: u32, vpn_only: bool) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let set_xray_status = self.set_xray_status;
        let base_url = api_base_url();
        let url = format!("{}/api/v1/control/pause", base_url);
        let seconds = minutes as u64 * 60;
        let payload = serde_json::json!({
            "duration": seconds,
            "onlySPN": vpn_only,
        });

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        let label = if vpn_only {
                            format!("VPN Paused for {}m", minutes)
                        } else {
                            format!("Paused for {}m", minutes)
                        };
                        set_paused.set(true);
                        set_pause_until.set(Some(label));
                        if vpn_only {
                            set_xray_status.update(|s| s.connected = false);
                            let disc_url = format!("{}/api/v1/xray/disconnect", base_url);
                            let _ = gloo_net::http::Request::post(&disc_url).json(&serde_json::json!({})).map(|r| r.send());
                        }
                    }
                }
            }
        });
    }

    /// Resumes all protection.
    pub fn resume(&self) {
        self.resume_protection();
    }

    /// Toggles VPN on/off.
    pub fn toggle_vpn(&self) {
        let status = self.xray_status.get_untracked();
        let set_status = self.set_xray_status;
        let set_busy = self.set_busy;
        let base_url = api_base_url();

        if status.connected {
            set_busy.set(true);
            let url = format!("{}/api/v1/xray/disconnect", base_url);
            let payload = serde_json::json!({});

            spawn_local(async move {
                if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(new_status) = resp.json::<XrayStatus>().await {
                                set_status.set(new_status);
                            } else {
                                set_status.update(|s| {
                                    s.connected = false;
                                    s.state = ProcessState::Stopped;
                                });
                            }
                        }
                    }
                }
                set_busy.set(false);
            });
        } else {
            let node_id = status
                .node_id
                .clone()
                .or_else(|| self.xray_nodes.get_untracked().first().map(|n| n.id.clone()))
                .unwrap_or_else(|| "node-nl-01".to_string());

            set_busy.set(true);
            let url = format!("{}/api/v1/xray/connect", base_url);
            let payload = serde_json::json!({ "nodeId": node_id });

            spawn_local(async move {
                if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(new_status) = resp.json::<XrayStatus>().await {
                                set_status.set(new_status);
                            } else {
                                set_status.update(|s| {
                                    s.node_id = Some(node_id);
                                    s.connected = true;
                                    s.state = ProcessState::Running;
                                });
                            }
                        }
                    }
                }
                set_busy.set(false);
            });
        }
    }

    /// Selects an active VPN node.
    pub fn select_node(&self, node_id: String) {
        let set_status = self.set_xray_status;
        let set_busy = self.set_busy;
        let base_url = api_base_url();
        let target_node = self.xray_nodes.get_untracked().into_iter().find(|n| n.id == node_id);

        set_busy.set(true);
        let url = format!("{}/api/v1/xray/connect", base_url);
        let payload = serde_json::json!({ "nodeId": node_id });

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Ok(new_status) = resp.json::<XrayStatus>().await {
                            set_status.set(new_status);
                        } else {
                            set_status.update(|s| {
                                s.node_id = Some(node_id.clone());
                                if let Some(target) = &target_node {
                                    s.node_name = Some(target.name.clone());
                                    s.country = Some(target.country_code.clone());
                                    s.exit_country = Some(target.country_code.clone());
                                }
                                s.connected = true;
                                s.state = ProcessState::Running;
                            });
                        }
                    }
                }
            }
            set_busy.set(false);
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

    /// Saves or removes a specific setting override for an application profile.
    pub fn save_profile_config(&self, app_id: &str, cfg_key: &str, value: Option<serde_json::Value>) {
        let app_id = app_id.to_string();
        let cfg_key = cfg_key.to_string();
        let client = self.client.clone();
        self.set_profiles.update(|list| {
            if let Some(p) = list.iter_mut().find(|p| p.id == app_id || format!("{}/{}", p.source, p.id) == app_id || (app_id.contains('/') && app_id.split('/').last() == Some(&p.id))) {
                let mut map = match p.config.take() {
                    Some(serde_json::Value::Object(m)) => m,
                    _ => serde_json::Map::new(),
                };
                if let Some(val) = value {
                    map.insert(cfg_key, val);
                } else {
                    map.remove(&cfg_key);
                }
                p.config = Some(serde_json::Value::Object(map));
                let db_key = p.database_key();
                let profile_val = serde_json::to_value(&*p).ok();
                spawn_local(async move {
                    if let Some(val) = profile_val {
                        let _ = client.put("update", &db_key, val).await;
                    }
                });
            }
        });
    }

    /// Explicitly permits a connection and updates its record and profile endpoint rule.
    pub fn allow_connection(&self, conn_id: &str) {
        let conn_id = conn_id.to_string();
        let mut target_domain = None;
        let mut target_profile = None;
        self.set_connections.update(|conns| {
            if let Some(c) = conns.iter_mut().find(|c| c.id == conn_id) {
                c.verdict = "permit".to_string();
                c.verdict_reason = "Manual Rule: Explicitly Allowed by User".to_string();
                target_domain = c.domain.clone().or_else(|| Some(c.remote_ip.clone()));
                target_profile = Some(c.profile_id.clone());
            }
        });
        if let (Some(domain), Some(prof_id)) = (target_domain, target_profile) {
            let client = self.client.clone();
            let rule = format!("+ {}", domain);
            self.set_profiles.update(|profs| {
                if let Some(p) = profs.iter_mut().find(|p| p.id == prof_id || p.id == prof_id.rsplit('/').next().unwrap_or(&prof_id)) {
                    let mut map = match p.config.take() {
                        Some(serde_json::Value::Object(m)) => m,
                        _ => serde_json::Map::new(),
                    };
                    let mut rules: Vec<String> = map.get("filter/endpoints")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    if !rules.contains(&rule) {
                        rules.push(rule.clone());
                        map.insert("filter/endpoints".to_string(), serde_json::json!(rules));
                        p.config = Some(serde_json::Value::Object(map.clone()));
                        let key = p.database_key();
                        let profile_val = serde_json::to_value(&*p).ok();
                        spawn_local(async move {
                            if let Some(val) = profile_val {
                                let _ = client.put("update", &key, val).await;
                            }
                        });
                    }
                }
            });
        }
    }

    /// Explicitly blocks a connection and updates its record and profile endpoint rule.
    pub fn block_connection(&self, conn_id: &str) {
        let conn_id = conn_id.to_string();
        let mut target_domain = None;
        let mut target_profile = None;
        self.set_connections.update(|conns| {
            if let Some(c) = conns.iter_mut().find(|c| c.id == conn_id) {
                c.verdict = "block".to_string();
                c.verdict_reason = "Manual Rule: Explicitly Blocked by User".to_string();
                target_domain = c.domain.clone().or_else(|| Some(c.remote_ip.clone()));
                target_profile = Some(c.profile_id.clone());
            }
        });
        if let (Some(domain), Some(prof_id)) = (target_domain, target_profile) {
            let client = self.client.clone();
            let rule = format!("- {}", domain);
            self.set_profiles.update(|profs| {
                if let Some(p) = profs.iter_mut().find(|p| p.id == prof_id || p.id == prof_id.rsplit('/').next().unwrap_or(&prof_id)) {
                    let mut map = match p.config.take() {
                        Some(serde_json::Value::Object(m)) => m,
                        _ => serde_json::Map::new(),
                    };
                    let mut rules: Vec<String> = map.get("filter/endpoints")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    if !rules.contains(&rule) {
                        rules.push(rule.clone());
                        map.insert("filter/endpoints".to_string(), serde_json::json!(rules));
                        p.config = Some(serde_json::Value::Object(map.clone()));
                        let key = p.database_key();
                        let profile_val = serde_json::to_value(&*p).ok();
                        spawn_local(async move {
                            if let Some(val) = profile_val {
                                let _ = client.put("update", &key, val).await;
                            }
                        });
                    }
                }
            });
        }
    }

    /// Requests graceful restart of Portmaster core service.
    pub fn restart_core(&self) {
        let url = format!("{}/api/v1/core/restart", api_base_url());
        spawn_local(async move {
            let _ = gloo_net::http::Request::post(&url).json(&serde_json::json!({})).map(|r| r.send());
        });
    }

    /// Requests shutdown of Portmaster core service.
    pub fn shutdown_core(&self) {
        let url = format!("{}/api/v1/core/shutdown", api_base_url());
        spawn_local(async move {
            let _ = gloo_net::http::Request::post(&url).json(&serde_json::json!({})).map(|r| r.send());
        });
    }

    /// Triggers updates check on Portmaster core.
    pub fn check_updates(&self) {
        let url = format!("{}/api/v1/updates/check", api_base_url());
        spawn_local(async move {
            let _ = gloo_net::http::Request::post(&url).json(&serde_json::json!({})).map(|r| r.send());
        });
    }

    /// Flushes resolver DNS cache.
    pub fn clear_dns_cache(&self) {
        let url = format!("{}/api/v1/dns/clear", api_base_url());
        spawn_local(async move {
            let _ = gloo_net::http::Request::post(&url).json(&serde_json::json!({})).map(|r| r.send());
        });
    }

    /// Reconnects or restarts Xray VPN connection.
    pub fn restart_xray(&self) {
        let status = self.xray_status.get_untracked();
        if let Some(node_id) = status.node_id {
            self.select_node(node_id);
        } else {
            self.toggle_vpn();
        }
    }

    /// Changes active security level and persists it to runtime registry.
    pub fn update_security_level(&self, level: &str) {
        self.set_security_level.set(level.to_string());
        let num_level = match level {
            "Untrusted" => 1,
            "Strict (Block All)" | "Strict" => 3,
            _ => 2, // Standard
        };
        let client = self.client.clone();
        spawn_local(async move {
            let _ = client
                .put(
                    "update",
                    "runtime:system/security-level",
                    serde_json::json!({
                        "SelectedSecurityLevel": num_level,
                    }),
                )
                .await;
        });
    }

    /// Clears network activity history.
    pub fn clear_history(&self) {
        let set_connections = self.set_connections;
        let url = format!("{}/api/v1/netquery/history/clear", api_base_url());
        let payload = serde_json::json!({});

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_connections.set(Vec::new());
                    }
                }
            }
        });
    }

    /// Refreshes Xray nodes list from core API.
    #[allow(dead_code)]
    pub fn refresh_xray_nodes(&self) {
        let set_xray_nodes = self.set_xray_nodes;
        let nodes_url = format!("{}/api/v1/xray/nodes", api_base_url());
        spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get(&nodes_url).send().await {
                if let Ok(nodes) = resp.json::<Vec<XrayNode>>().await {
                    set_xray_nodes.set(nodes);
                }
            }
        });
    }

    /// Adds a subscription URL and refreshes nodes on success.
    pub fn add_xray_subscription(&self, url: String) {
        let set_xray_nodes = self.set_xray_nodes;
        let base_url = api_base_url();
        let sub_url = format!("{}/api/v1/xray/subscriptions", base_url);
        let nodes_url = format!("{}/api/v1/xray/nodes", base_url);
        let payload = serde_json::json!({ "url": url });

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&sub_url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        if let Ok(nodes_resp) = gloo_net::http::Request::get(&nodes_url).send().await {
                            if let Ok(nodes) = nodes_resp.json::<Vec<XrayNode>>().await {
                                set_xray_nodes.set(nodes);
                            }
                        }
                    }
                }
            }
        });
    }
}
