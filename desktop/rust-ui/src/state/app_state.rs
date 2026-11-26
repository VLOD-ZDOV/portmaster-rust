//! Global application state and reactive signals for Portmaster UI.

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;
use pm_models::profile::AppProfile;
use pm_models::settings::{ExpertiseLevel, Setting};
use pm_models::xray::{ProcessState, XrayNode, XrayStatus};
use wasm_bindgen_futures::spawn_local;

use crate::client::PortApiClient;

/// Helper to determine the API base URL from window location or default to 127.0.0.1:817.
/// Accepts any port on loopback hosts (the core may listen on a custom port);
/// this mirrors the WebSocket endpoint selection in `client::ws`.
pub fn api_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let loc = window.location();
        if let Ok(host) = loc.host() {
            let is_loopback = host.starts_with("127.0.0.1:")
                || host.starts_with("localhost:")
                || host.starts_with("[::1]:");
            if is_loopback {
                let proto = loc.protocol().unwrap_or_else(|_| "http:".to_string());
                return format!("{}//{}", proto, host);
            }
        }
    }
    "http://127.0.0.1:817".to_string()
}

/// Formats a netquery `Started` timestamp as an age string.
fn relative_time(started: &Option<String>) -> String {
    let Some(started) = started else {
        return String::new();
    };
    // The HTTP netquery path serializes SQLite time as `YYYY-MM-DD HH:MM:SS`
    // in UTC; JavaScript would otherwise parse it as local time.
    let normalized = if started.contains('T') {
        started.clone()
    } else {
        format!("{}Z", started.replace(' ', "T"))
    };
    let then = js_sys::Date::parse(&normalized);
    if !then.is_finite() {
        return String::new();
    }
    let seconds = ((js_sys::Date::now() - then) / 1000.0).max(0.0) as u64;
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", seconds / 60),
        3600..=86399 => format!("{}h ago", seconds / 3600),
        _ => format!("{}d ago", seconds / 86400),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(Default)]
pub enum ViewTab {
    #[default]
    Apps,
    Vpn,
    Settings,
    Monitor,
}


/// Navigation pages for Portmaster UI.
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Default)]
pub enum Page {
    Monitor,
    #[default]
    Apps,
    AppDetail(String),
    Xray,
    Settings,
    Support,
}


/// A notification item for the notification drawer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub level: &'static str, // "info", "warning", "error"
    pub time: &'static str,
    /// Whether this notification is an interactive connection prompt.
    pub is_prompt: bool,
    /// Unix expiry time, if the notification has one.
    pub expires_at: Option<i64>,
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
    pub vpn_paused: ReadSignal<bool>,
    pub set_vpn_paused: WriteSignal<bool>,
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
        let (vpn_paused, set_vpn_paused) = signal(false);
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
            vpn_paused,
            set_vpn_paused,
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
                    "bytes_sent", "bytes_received", "active", "started"
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
                        for conn in &mut conns {
                            if conn.time_ago.is_empty() {
                                conn.time_ago = relative_time(&conn.started);
                            }
                        }
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
                        if !key_str.is_empty()
                            && (p.id == key_str || p.id == key_id || key_str.ends_with(&format!("/{}", p.id))) {
                                return false;
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
        let set_route_default = self.set_route_by_default;
        let set_sec_level = self.set_security_level;
        client.qsub("config:", move |frame| {
            if frame.msg_type == "del" {
                if let Some(key) = &frame.key {
                    let setting_key = key.strip_prefix("config:").unwrap_or(key);
                    set_settings.update(|list| {
                        list.retain(|s| s.key != setting_key);
                    });
                    match setting_key {
                        "spn/use" => set_route_default.set(false),
                        "filter/defaultAction" => {
                            set_sec_level.set("Standard (Balanced)".to_string());
                        }
                        _ => {}
                    }
                }
            } else if let Ok(setting) = frame.parse_data::<Setting>() {
                // Keep the quick toggles in sync with the real config,
                // including the initial qsub snapshot.
                let effective = setting.value.clone().unwrap_or_else(|| setting.default_value.clone());
                match setting.key.as_str() {
                    "spn/use" => {
                        set_route_default.set(effective.as_bool().unwrap_or(false));
                    }
                    "filter/defaultAction" => {
                        let label = match effective.as_str().unwrap_or("permit") {
                            "ask" => "Untrusted",
                            "block" => "Strict (Block All)",
                            _ => "Standard (Balanced)",
                        };
                        set_sec_level.set(label.to_string());
                    }
                    _ => {}
                }
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
        let notif_read = self.notifications;
        let notif_prompts = self.set_pending_prompts;
        let control_set_paused = self.set_paused;
        let control_set_vpn_paused = self.set_vpn_paused;
        let control_set_pause_until = self.set_pause_until;
        client.qsub("notifications:all/", move |frame| {
            if frame.msg_type == "del" {
                if let Some(key) = &frame.key {
                    let id_to_remove = key.strip_prefix("notifications:all/").unwrap_or(key);
                    notif_set.update(|list: &mut Vec<AppNotification>| {
                        list.retain(|n| n.id != id_to_remove);
                    });
                    if id_to_remove == "control:paused" || id_to_remove == "control:resumed" {
                        control_set_paused.set(false);
                        control_set_vpn_paused.set(false);
                        control_set_pause_until.set(None);
                    }
                }
            } else if let Ok(val) = frame.parse_data::<serde_json::Value>() {
                let id = val.get("EventID").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = val.get("Title").and_then(|v| v.as_str()).unwrap_or("Notification").to_string();
                let message = val.get("Message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                // Notification.Type is a numeric enum: 0 info, 1 warning,
                // 2 prompt, 3 error (pm-notifications::TYPE_*).
                let notif_type = val.get("Type").and_then(|v| v.as_u64()).unwrap_or(0);
                let is_prompt = notif_type == 2;
                let level = match notif_type {
                    1 | 2 => "warning",
                    3 => "error",
                    _ => "info",
                };
                let state = val
                    .get("State")
                    .and_then(|value| value.as_str())
                    .unwrap_or("active");
                let selected = val
                    .get("SelectedActionID")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let expires_at = val
                    .get("Expires")
                    .and_then(|value| value.as_i64())
                    .filter(|value| *value > 0);
                let now = (js_sys::Date::now() / 1000.0) as i64;
                let active_prompt = is_prompt
                    && state == "active"
                    && selected.is_empty()
                    && expires_at.map(|expires| expires > now).unwrap_or(true);
                if !id.is_empty() {
                    if is_prompt && !active_prompt {
                        // Answered, cancelled or expired prompts must disappear
                        // immediately; otherwise the badge stays forever.
                        notif_set.update(|list: &mut Vec<AppNotification>| {
                            list.retain(|item| item.id != id);
                        });
                    } else {
                        notif_set.update(|list: &mut Vec<AppNotification>| {
                            let item = AppNotification {
                                id: id.clone(),
                                title,
                                message,
                                level,
                                time: "Now",
                                is_prompt: active_prompt,
                                expires_at,
                            };
                            if let Some(pos) = list.iter().position(|n| n.id == id) {
                                list[pos] = item;
                            } else {
                                list.push(item);
                            }
                        });
                    }
                }
                // Keep the Pause state in sync with the control module,
                // including the initial qsub snapshot and automatic resume.
                match id.as_str() {
                    "control:paused" => {
                        let event = val.get("EventData");
                        let interception = event
                            .and_then(|data| data.get("Interception"))
                            .and_then(|value| value.as_bool())
                            .unwrap_or(true);
                        let spn = event
                            .and_then(|data| data.get("SPN"))
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false);
                        control_set_paused.set(interception || spn);
                        control_set_vpn_paused.set(spn && !interception);
                        let until = event
                            .and_then(|data| data.get("TillTime"))
                            .and_then(|value| value.as_str())
                            .map(|value| {
                                let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
                                if date.get_time().is_nan() {
                                    "Paused".to_string()
                                } else {
                                    format!(
                                        "Paused until {:02}:{:02}",
                                        date.get_hours(),
                                        date.get_minutes()
                                    )
                                }
                            });
                        control_set_pause_until.set(until);
                    }
                    "control:resumed" => {
                        control_set_paused.set(false);
                        control_set_vpn_paused.set(false);
                        control_set_pause_until.set(None);
                    }
                    _ => {}
                }
            }
            // Derive the prompt badge from the list so updates of existing
            // prompts do not inflate it and deletions deflated it.
            let count = notif_read.get_untracked().iter().filter(|n| n.is_prompt).count();
            notif_prompts.set(count);
        });

        // 7. Subscribe to real-time connection events via netquery/data/
        let live_set_conns = set_connections;
        let live_set_profiles = set_profiles;
        client.qsub("netquery/data/", move |frame| {
            if let Ok(mut conn) = frame.parse_data::<ConnectionRecord>() {
                if conn.time_ago.is_empty() {
                    conn.time_ago = relative_time(&conn.started);
                }
                let c_clone = conn.clone();
                let mut prev_sent = 0u64;
                let mut prev_recv = 0u64;
                let mut prev_active = false;
                let mut is_new = false;

                live_set_conns.update(|list| {
                    if let Some(pos) = list.iter().position(|c| c.id == conn.id) {
                        prev_sent = list[pos].bytes_sent;
                        prev_recv = list[pos].bytes_received;
                        prev_active = list[pos].active;
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
                        if c_clone.active != prev_active {
                            if c_clone.active {
                                stats.connections_count = stats.connections_count.saturating_add(1);
                            } else {
                                stats.connections_count = stats.connections_count.saturating_sub(1);
                            }
                        }
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

    /// Adds a local UI notification (e.g. a failed action) to the drawer.
    pub fn notify_local(&self, title: &str, message: &str, level: &'static str) {
        let id = format!(
            "ui-{}",
            js_sys::Date::now() as u64
        );
        self.set_notifications.update(|list| {
            list.push(AppNotification {
                id,
                title: title.to_string(),
                message: message.to_string(),
                level,
                time: "Now",
                is_prompt: false,
                expires_at: None,
            });
        });
    }

    /// Clears non-prompt notifications from the drawer. Active connection
    /// prompts stay visible: hiding them would strand the waiting connection
    /// until the core prompt timeout.
    pub fn clear_notifications(&self) {
        let now = (js_sys::Date::now() / 1000.0) as i64;
        self.set_notifications.update(|list| {
            list.retain(|item| {
                item.is_prompt
                    && item
                        .expires_at
                        .map(|expires| expires > now)
                        .unwrap_or(true)
            })
        });
        let count = self
            .notifications
            .get_untracked()
            .iter()
            .filter(|item| item.is_prompt)
            .count();
        self.set_pending_prompts.set(count);
    }

    /// Persists a profile change, rolling back the optimistic UI update and
    /// notifying the user when the core rejects the write.
    fn persist_profile(&self, previous: AppProfile, key: String, val: serde_json::Value) {
        let client = self.client.clone();
        let set_profiles = self.set_profiles;
        let notifications = self.set_notifications;
        spawn_local(async move {
            if let Err(error) = client.put("update", &key, val).await {
                set_profiles.update(|list| {
                    if let Some(pos) = list.iter().position(|p| p.id == previous.id) {
                        list[pos] = previous;
                    }
                });
                let id = format!("ui-{}", js_sys::Date::now() as u64);
                notifications.update(|list| {
                    list.push(AppNotification {
                        id,
                        title: "Profile change rejected".to_string(),
                        message: error,
                        level: "error",
                        time: "Now",
                        is_prompt: false,
                        expires_at: None,
                    });
                });
            }
        });
    }

    /// Toggles "Route Through VPN (Xray) by Default" global setting.
    /// This is `spn/use` (the same option the Angular UI edits), not the Split
    /// Tunnel option. The VPN must already be connected separately.
    pub fn toggle_route_by_default(&self) {
        let current = self.route_by_default.get_untracked();
        let new_val = !current;
        let set_route_by_default = self.set_route_by_default;
        let state = self.clone();

        let client = self.client.clone();
        spawn_local(async move {
            let result = client
                .put(
                    "update",
                    "config:spn/use",
                    serde_json::json!({
                        "Key": "spn/use",
                        "Value": new_val
                    }),
                )
                .await;
            if let Err(error) = result {
                state.notify_local(
                    "VPN routing",
                    &format!("Failed to change the default VPN routing setting: {error}"),
                    "error",
                );
                return;
            }
            set_route_by_default.set(new_val);
        });
    }

    /// Pauses filtering for the given duration in seconds.
    #[allow(dead_code)]
    pub fn pause_protection(&self, seconds: u64) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let set_vpn_paused = self.set_vpn_paused;
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
                        set_vpn_paused.set(false);
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
        let set_vpn_paused = self.set_vpn_paused;
        let url = format!("{}/api/v1/control/resume", api_base_url());
        let payload = serde_json::json!({});
        let state = self.clone();

        spawn_local(async move {
            let mut ok = false;
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        ok = true;
                        set_paused.set(false);
                        set_vpn_paused.set(false);
                        set_pause_until.set(None);
                    }
                }
            }
            if !ok {
                state.notify_local(
                    "Resume",
                    "The core rejected the resume request.",
                    "error",
                );
            }
        });
    }

    /// Saves a setting value to core, rolling back on rejection.
    pub fn save_setting(&self, key: impl Into<String>, value: Option<serde_json::Value>) {
        let key = key.into();
        let set_settings = self.set_settings;
        let client = self.client.clone();
        let state = self.clone();

        let mut previous: Option<Option<serde_json::Value>> = None;
        let mut db_key = String::new();
        set_settings.update(|list| {
            if let Some(s) = list.iter_mut().find(|item| item.key == key) {
                previous = Some(s.value.clone());
                s.value = value.clone();
                db_key = s.database_key();
            }
        });
        let Some(previous) = previous else { return };

        spawn_local(async move {
            let result = if let Some(val) = value {
                client
                    .put(
                        "update",
                        &db_key,
                        serde_json::json!({
                            "Key": key,
                            "Value": val,
                        }),
                    )
                    .await
            } else {
                client.delete(&db_key).await
            };
            if let Err(error) = result {
                set_settings.update(|list| {
                    if let Some(s) = list.iter_mut().find(|item| item.key == key) {
                        s.value = previous;
                    }
                });
                state.notify_local("Setting not saved", &error, "error");
            }
        });
    }

    /// Updates a setting with a new value.
    pub fn update_setting(&self, key: impl Into<String>, value: serde_json::Value) {
        self.save_setting(key, Some(value));
    }

    /// Resets a setting to default by deleting its override.
    pub fn reset_setting(&self, key: impl Into<String>) {
        self.save_setting(key, None);
    }

    /// Pauses protection for specified duration in minutes.
    pub fn pause(&self, minutes: u32, vpn_only: bool) {
        let set_paused = self.set_paused;
        let set_pause_until = self.set_pause_until;
        let set_vpn_paused = self.set_vpn_paused;
        let base_url = api_base_url();
        let url = format!("{}/api/v1/control/pause", base_url);
        let seconds = minutes as u64 * 60;
        let payload = serde_json::json!({
            "duration": seconds,
            "onlySPN": vpn_only,
        });
        let state = self.clone();

        spawn_local(async move {
            let mut ok = false;
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        ok = true;
                        let label = if vpn_only {
                            format!("VPN Paused for {}m", minutes)
                        } else {
                            format!("Paused for {}m", minutes)
                        };
                        set_paused.set(true);
                        set_vpn_paused.set(vpn_only);
                        set_pause_until.set(Some(label));
                    }
                }
            }
            if !ok {
                state.notify_local(
                    "Pause",
                    "The core rejected the pause request.",
                    "error",
                );
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
            let state = self.clone();

            spawn_local(async move {
                let mut ok = false;
                if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            ok = true;
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
                if !ok {
                    state.notify_local("VPN", "Failed to disconnect the VPN.", "error");
                }
                set_busy.set(false);
            });
        } else {
            let Some(node_id) = status
                .node_id
                .clone()
                .or_else(|| self.xray_nodes.get_untracked().first().map(|n| n.id.clone()))
            else {
                self.notify_local(
                    "VPN",
                    "No VPN nodes available. Add a subscription or configure a node first.",
                    "warning",
                );
                return;
            };

            set_busy.set(true);
            let url = format!("{}/api/v1/xray/connect", base_url);
            let payload = serde_json::json!({ "nodeId": node_id });
            let state = self.clone();

            spawn_local(async move {
                let mut ok = false;
                if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            ok = true;
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
                if !ok {
                    state.notify_local("VPN", "Failed to connect to the VPN node.", "error");
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
        let state = self.clone();

        spawn_local(async move {
            let mut ok = false;
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        ok = true;
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
            if !ok {
                state.notify_local("VPN", "Failed to connect to the selected node.", "error");
            }
            set_busy.set(false);
        });
    }

    /// Helper to mutate an `AppProfile` optimistically and persist via `persist_profile`.
    fn mutate_profile(&self, app_id: &str, mutate: impl FnOnce(&mut AppProfile)) {
        let mut change = None;
        self.set_profiles.update(|list| {
            if let Some(p) = list.iter_mut().find(|item| {
                item.id == app_id
                    || format!("{}/{}", item.source, item.id) == app_id
                    || (app_id.contains('/') && app_id.split('/').next_back() == Some(&item.id))
            }) {
                let previous = p.clone();
                mutate(p);
                let key = p.database_key();
                if let Ok(val) = serde_json::to_value(&*p) {
                    change = Some((previous, key, val));
                }
            }
        });
        if let Some((previous, key, val)) = change {
            self.persist_profile(previous, key, val);
        }
    }

    /// Sets or clears an application's explicit VPN override (`None` = inherit global).
    pub fn set_app_vpn_override(&self, app_id: String, override_val: Option<bool>) {
        self.mutate_profile(&app_id, |p| p.set_vpn_route_override(override_val));
    }

    /// Sets or clears an application's explicit Split Tunnel override (`None` = inherit global).
    pub fn set_app_split_tun_override(&self, app_id: String, override_val: Option<bool>) {
        self.mutate_profile(&app_id, |p| p.set_split_tun_override(override_val));
    }

    /// Cycles an application's VPN override: Inherit Global -> Always VPN -> Always Direct -> Inherit Global.
    pub fn toggle_app_vpn(&self, app_id: String) {
        self.mutate_profile(&app_id, |p| {
            let next = match p.vpn_route_override() {
                None => Some(true),
                Some(true) => Some(false),
                Some(false) => None,
            };
            p.set_vpn_route_override(next);
        });
    }

    /// Changes an application default filter action ("permit", "block", "ask").
    pub fn set_app_action(&self, app_id: String, action: String) {
        self.mutate_profile(&app_id, |p| p.set_default_action(&action));
    }

    /// Saves or removes a specific setting override for an application profile.
    pub fn save_profile_config(&self, app_id: &str, cfg_key: &str, value: Option<serde_json::Value>) {
        let cfg_key = cfg_key.to_string();
        self.mutate_profile(app_id, |p| {
            let mut map = match p.config.take() {
                Some(serde_json::Value::Object(m)) => m,
                _ => serde_json::Map::new(),
            };
            if let Some(val) = value {
                map.insert(cfg_key, val);
            } else {
                map.remove(&cfg_key);
            }
            p.config = if map.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(map))
            };
        });
    }

    /// Clears `spn/use` and `splittun/use` overrides across all local profiles via core HTTP API.
    pub fn reset_all_routing_overrides(&self) {
        let base_url = api_base_url();
        let url = format!("{}/api/v1/profiles/reset-routing", base_url);
        let set_profiles = self.set_profiles;
        let state = self.clone();

        spawn_local(async move {
            let Ok(req) = gloo_net::http::Request::post(&url).json(&serde_json::json!({})) else {
                state.notify_local("VPN routing", "Failed to build reset request.", "error");
                return;
            };
            match req.send().await {
                Ok(resp) if resp.ok() => {
                    let updated = resp
                        .json::<serde_json::Value>()
                        .await
                        .ok()
                        .and_then(|v| v.get("updated").and_then(|n| n.as_u64()))
                        .unwrap_or(0);
                    set_profiles.update(|list| {
                        for p in list.iter_mut() {
                            if p.source == "local" {
                                p.set_vpn_route_override(None);
                                p.set_split_tun_override(None);
                            }
                        }
                    });
                    state.notify_local(
                        "VPN routing",
                        &format!("Reset {updated} app profiles to global defaults."),
                        "info",
                    );
                }
                _ => {
                    state.notify_local("VPN routing", "Failed to reset app routing overrides.", "error");
                }
            }
        });
    }

    /// Resolves a pending connection prompt by selecting the matching action
    /// on its notification, like the Angular UI. Returns `false` when there is
    /// no matching prompt, so the caller can create the rule directly.
    fn resolve_prompt(&self, prof_id: &str, domain: &str, ip: &str, inbound: bool, allow: bool) -> bool {
        let bare_prof = prof_id.rsplit('/').next().unwrap_or(prof_id);
        let mut candidates = Vec::new();
        if inbound || domain.is_empty() {
            candidates.push(format!("filter:prompt-{bare_prof}-{inbound}-{ip}"));
            if bare_prof != prof_id {
                candidates.push(format!("filter:prompt-{prof_id}-{inbound}-{ip}"));
            }
        } else {
            candidates.push(format!("filter:prompt-{bare_prof}-{domain}"));
            if bare_prof != prof_id {
                candidates.push(format!("filter:prompt-{prof_id}-{domain}"));
            }
        }

        let notifications = self.notifications.get_untracked();
        let Some(prompt) = notifications
            .iter()
            .find(|n| n.is_prompt && candidates.iter().any(|candidate| candidate == &n.id))
        else {
            return false;
        };
        let action = match (inbound, domain.is_empty(), allow) {
            (true, _, true) => "allow-serving-ip",
            (true, _, false) => "block-serving-ip",
            (false, true, true) => "allow-ip",
            (false, true, false) => "block-ip",
            (false, false, true) => "allow-domain-all",
            (false, false, false) => "block-domain-all",
        };
        let client = self.client.clone();
        let set_notifications = self.set_notifications;
        let prompt_id = prompt.id.clone();
        let key = format!("notifications:all/{prompt_id}");
        let payload = serde_json::json!({
            "EventID": prompt_id,
            "SelectedActionID": action,
        });
        spawn_local(async move {
            if let Err(error) = client.put("update", &key, payload).await {
                let id = format!("ui-{}", js_sys::Date::now() as u64);
                set_notifications.update(|list| {
                    list.push(AppNotification {
                        id,
                        title: "Connection prompt".to_string(),
                        message: format!("Failed to send the decision to the core: {error}"),
                        level: "error",
                        time: "Now",
                        is_prompt: false,
                        expires_at: None,
                    });
                });
            }
        });
        true
    }

    /// Applies a manual allow/block decision to a connection. When the core is
    /// waiting for a prompt answer, only the notification is updated: the core
    /// saves the correct endpoint rule itself. Otherwise a rule is added to the
    /// profile so the decision applies to future connections.
    fn set_connection_decision(&self, conn_id: &str, allow: bool) {
        let mut decision = None;
        self.set_connections.update(|conns| {
            if let Some(c) = conns.iter_mut().find(|c| c.id == conn_id) {
                c.verdict = if allow { "permit" } else { "block" }.to_string();
                c.verdict_reason = if allow {
                    "Manual Rule: Explicitly Allowed by User"
                } else {
                    "Manual Rule: Explicitly Blocked by User"
                }
                .to_string();
                decision = Some((
                    c.profile_id.clone(),
                    c.domain.clone().unwrap_or_default(),
                    c.remote_ip.clone(),
                    c.direction == "inbound",
                ));
            }
        });

        let Some((profile_id, domain, ip, inbound)) = decision else {
            return;
        };
        if self.resolve_prompt(&profile_id, &domain, &ip, inbound, allow) {
            return;
        }
        self.add_endpoint_rule(&profile_id, &domain, &ip, inbound, allow);
    }

    /// Adds an endpoint rule to the profile when no core prompt was waiting.
    fn add_endpoint_rule(&self, profile_id: &str, domain: &str, ip: &str, inbound: bool, allow: bool) {
        let (rule, service) = match (inbound, domain.is_empty(), allow) {
            (true, _, true) => (format!("+ {ip}"), true),
            (true, _, false) => (format!("- {ip}"), true),
            (false, true, true) => (format!("+ {ip}"), false),
            (false, true, false) => (format!("- {ip}"), false),
            (false, false, true) => (format!("+ .{domain}"), false),
            (false, false, false) => (format!("- .{domain}"), false),
        };
        if rule.trim_end() == "+" || rule.trim_end() == "-" {
            return;
        }
        let list_key = if service {
            "filter/serviceEndpoints"
        } else {
            "filter/endpoints"
        };
        let bare = profile_id.rsplit('/').next().unwrap_or(profile_id);
        let mut change = None;
        self.set_profiles.update(|profs| {
            if let Some(p) = profs.iter_mut().find(|p| p.id == profile_id || p.id == bare) {
                let previous = p.clone();
                let mut map = match p.config.take() {
                    Some(serde_json::Value::Object(m)) => m,
                    _ => serde_json::Map::new(),
                };
                let mut rules: Vec<String> = map
                    .get(list_key)
                    .and_then(|value| serde_json::from_value(value.clone()).ok())
                    .unwrap_or_default();
                if !rules.contains(&rule) {
                    rules.push(rule.clone());
                    map.insert(list_key.to_string(), serde_json::json!(rules));
                    p.config = Some(serde_json::Value::Object(map));
                    let key = p.database_key();
                    if let Ok(value) = serde_json::to_value(&*p) {
                        change = Some((previous, key, value));
                    }
                }
            }
        });
        if let Some((previous, key, value)) = change {
            self.persist_profile(previous, key, value);
        }
    }

    /// Explicitly permits a connection and updates its record and profile endpoint rule.
    pub fn allow_connection(&self, conn_id: &str) {
        self.set_connection_decision(conn_id, true);
    }

    /// Explicitly blocks a connection and updates its record and profile endpoint rule.
    pub fn block_connection(&self, conn_id: &str) {
        self.set_connection_decision(conn_id, false);
    }

    /// Requests graceful restart of Portmaster core service.
    pub fn restart_core(&self) {
        let url = format!("{}/api/v1/core/restart", api_base_url());
        let state = self.clone();
        spawn_local(async move {
            match gloo_net::http::Request::post(&url).json(&serde_json::json!({})) {
                Ok(req) => match req.send().await {
                    Ok(resp) if resp.ok() => {}
                    _ => state.notify_local("Core restart", "The core rejected the restart request.", "error"),
                },
                Err(_) => state.notify_local("Core restart", "Failed to send the restart request.", "error"),
            }
        });
    }

    /// Requests shutdown of Portmaster core service.
    pub fn shutdown_core(&self) {
        let url = format!("{}/api/v1/core/shutdown", api_base_url());
        let state = self.clone();
        spawn_local(async move {
            match gloo_net::http::Request::post(&url).json(&serde_json::json!({})) {
                Ok(req) => match req.send().await {
                    Ok(resp) if resp.ok() => {}
                    _ => state.notify_local("Core shutdown", "The core rejected the shutdown request.", "error"),
                },
                Err(_) => state.notify_local("Core shutdown", "Failed to send the shutdown request.", "error"),
            }
        });
    }

    /// Triggers updates check on Portmaster core.
    pub fn check_updates(&self) {
        let url = format!("{}/api/v1/updates/check", api_base_url());
        let state = self.clone();
        spawn_local(async move {
            match gloo_net::http::Request::post(&url).json(&serde_json::json!({})) {
                Ok(req) => match req.send().await {
                    Ok(resp) if resp.ok() => {}
                    _ => state.notify_local("Updates", "The update check request failed.", "error"),
                },
                Err(_) => state.notify_local("Updates", "Failed to send the update check request.", "error"),
            }
        });
    }

    /// Flushes resolver DNS cache.
    pub fn clear_dns_cache(&self) {
        let url = format!("{}/api/v1/dns/clear", api_base_url());
        let state = self.clone();
        spawn_local(async move {
            match gloo_net::http::Request::post(&url).json(&serde_json::json!({})) {
                Ok(req) => match req.send().await {
                    Ok(resp) if resp.ok() => {}
                    _ => state.notify_local("DNS cache", "Failed to clear the DNS cache.", "error"),
                },
                Err(_) => state.notify_local("DNS cache", "Failed to send the cache clear request.", "error"),
            }
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

    /// Changes active security level by setting the global default filter
    /// action: Standard permits, Untrusted asks, Strict blocks everything.
    pub fn update_security_level(&self, level: &str) {
        let previous = self.security_level.get_untracked();
        self.set_security_level.set(level.to_string());
        let action = match level {
            "Untrusted" => "ask",
            "Strict (Block All)" | "Strict" => "block",
            _ => "permit", // Standard
        };
        let client = self.client.clone();
        let set_level = self.set_security_level;
        let state = self.clone();
        spawn_local(async move {
            if client
                .put(
                    "update",
                    "config:filter/defaultAction",
                    serde_json::json!({
                        "Key": "filter/defaultAction",
                        "Value": action,
                    }),
                )
                .await
                .is_err()
            {
                set_level.set(previous);
                state.notify_local(
                    "Security level",
                    "The core rejected the security level change.",
                    "error",
                );
            }
        });
    }

    /// Clears network activity history.
    pub fn clear_history(&self) {
        let set_connections = self.set_connections;
        let url = format!("{}/api/v1/netquery/history/clear", api_base_url());
        let payload = serde_json::json!({});
        let state = self.clone();

        spawn_local(async move {
            if let Ok(req) = gloo_net::http::Request::post(&url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        set_connections.set(Vec::new());
                        return;
                    }
                }
            }
            state.notify_local("History", "Failed to clear the network history.", "error");
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
        let state = self.clone();

        spawn_local(async move {
            let mut ok = false;
            if let Ok(req) = gloo_net::http::Request::post(&sub_url).json(&payload) {
                if let Ok(resp) = req.send().await {
                    if resp.ok() {
                        ok = true;
                        if let Ok(nodes_resp) = gloo_net::http::Request::get(&nodes_url).send().await {
                            if let Ok(nodes) = nodes_resp.json::<Vec<XrayNode>>().await {
                                set_xray_nodes.set(nodes);
                            }
                        }
                    }
                }
            }
            if !ok {
                state.notify_local(
                    "VPN subscription",
                    "Failed to add the subscription. Check the URL and the core log.",
                    "error",
                );
            }
        });
    }
}
