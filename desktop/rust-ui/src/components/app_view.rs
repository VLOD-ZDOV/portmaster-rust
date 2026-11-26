//! Detailed Application View matching Portmaster 1:1.
//!
//! Replicates Portmaster's application details view:
//! - Breadcrumbs & expertise selector
//! - App header with 3rem monogram icon, metadata, and quick settings
//! - Bandwidth / traffic counters (Received / Sent)
//! - 4 Tabs: Connections, Rules & Routing, App Settings Overrides, Details

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;
use pm_models::profile::{FingerprintOperation, FingerprintType};

use crate::i18n::t;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Connections,
    Rules,
    Settings,
    Details,
}

#[component]
pub fn AppDetailView(app_id: String) -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let connections = state.connections;
    let (active_tab, set_active_tab) = signal(AppTab::Connections);

    // Endpoint rules list local state & path tester (#2166)
    let (new_rule, set_new_rule) = signal(String::new());
    let (new_rule_action, set_new_rule_action) = signal("block".to_string());
    let (test_path, set_test_path) = signal(String::new());
    let (confirm_delete, set_confirm_delete) = signal(false);
    let lang = state.lang;

    let state_back = state.clone();
    let on_back = move |_| {
        state_back.select_app(None);
    };

    let id_clone = app_id.clone();
    let profile = Memo::new(move |_| {
        let list = profiles.get();
        list.into_iter().find(|p| {
            p.id == id_clone
                || format!("{}/{}", p.source, p.id) == id_clone
                || (id_clone.contains('/') && id_clone.split('/').next_back() == Some(&p.id))
        })
    });

    view! {
        <div class="space-y-4">
            // Breadcrumbs Header + Export/Delete Profile (#388)
            <div class="flex flex-wrap items-center justify-between gap-2 pb-2 border-b border-[#2c2c2c] text-xs">
                <div class="flex items-center space-x-1.5 text-[#888888]">
                    <button
                        on:click=on_back
                        class="hover:text-white cursor-pointer transition-colors flex items-center space-x-1 font-medium"
                    >
                        <span>{move || t(lang.get(), "app.back")}</span>
                    </button>
                    <span>"/"</span>
                    <span class="text-white font-medium">
                        {move || profile.get().map(|p| p.display_name().to_string()).unwrap_or_else(|| app_id.clone())}
                    </span>
                </div>

                <div class="flex items-center gap-2">
                    {move || {
                        let s_exp = expect_context::<AppState>();
                        let s_del = expect_context::<AppState>();
                        let cur = profile.get();
                        cur.map(|p| {
                            let scoped = p.database_key().trim_start_matches("core:profiles/").to_string();
                            let pid_del = p.id.clone();
                            view! {
                                <button
                                    on:click=move |_| s_exp.export_profile_sync(scoped.clone())
                                    class="btn-pm !bg-[#222222] text-[#4e97fa] border border-[#2c2c2c] text-[11px]"
                                >
                                    {t(lang.get(), "app.export")}
                                </button>
                                <button
                                    on:click=move |_| {
                                        if confirm_delete.get() {
                                            s_del.delete_profile(pid_del.clone());
                                        } else {
                                            set_confirm_delete.set(true);
                                        }
                                    }
                                    class=move || if confirm_delete.get() {
                                        "btn-pm !bg-[#811818] text-white border border-[#e01d1d] text-[11px]"
                                    } else {
                                        "btn-pm !bg-[#3d1414] text-[#e01d1d] border border-[#e01d1d]/30 text-[11px]"
                                    }
                                >
                                    {move || if confirm_delete.get() { "Confirm Delete" } else { t(lang.get(), "app.delete_profile") }}
                                </button>
                            }
                        })
                    }}
                </div>
            </div>

            {move || {
                let state = expect_context::<AppState>();
                let Some(p) = profile.get() else {
                    return view! {
                        <div class="p-12 text-center rounded bg-[#1b1b1b] border border-[#2c2c2c] text-[#888888]">
                            "Application profile not found."
                        </div>
                    }.into_any();
                };

                let app_id = p.id.clone();
                let global_vpn = state.route_by_default.get();
                let vpn_override = p.vpn_route_override();
                let is_vpn = p.route_via_vpn_with_default(global_vpn);
                let split_override = p.split_tun_override();
                let global_split = state
                    .settings
                    .get()
                    .into_iter()
                    .find(|s| s.key == "splittun/use")
                    .and_then(|s| s.effective_value().as_bool())
                    .unwrap_or(false);
                let action = p.default_action().to_string();
                let display_name = p.display_name().to_string();
                let display_name_match = display_name.clone();
                let path = if !p.presentation_path.is_empty() { p.presentation_path.clone() } else { p.source.clone() };
                let monogram = display_name.chars().next().unwrap_or('A').to_uppercase().to_string();
                let conn_count = p.count_alive_connections();

                let conns = connections.get();
                let app_id_curr = app_id.clone();
                let app_name_curr = display_name_match.clone();
                let app_conns_summary: Vec<_> = conns
                    .iter()
                    .filter(|c| {
                        c.profile_id == app_id_curr
                            || c.profile_id.ends_with(&format!("/{}", app_id_curr))
                            || (!app_name_curr.is_empty() && c.profile_name.eq_ignore_ascii_case(&app_name_curr))
                    })
                    .collect();

                let mut conn_recv = 0u64;
                let mut conn_sent = 0u64;
                let mut conn_alive = 0usize;
                let mut conn_blocked = 0usize;
                for c in &app_conns_summary {
                    conn_recv = conn_recv.saturating_add(c.bytes_received);
                    conn_sent = conn_sent.saturating_add(c.bytes_sent);
                    if c.active {
                        conn_alive += 1;
                    }
                    if c.is_blocked() {
                        conn_blocked += 1;
                    }
                }

                let (alive_conns, blocked_pct, bytes_recv, bytes_sent) = if let Some(s) = p.stats {
                    let r = s.bytes_received.max(conn_recv);
                    let sent = s.bytes_sent.max(conn_sent);
                    let alive = if s.connections_count > 0 { s.connections_count } else { conn_alive as u64 };
                    // Derive the percentage from the current connection list;
                    // profile stats are cumulative and can double count pushes.
                    let pct = if !app_conns_summary.is_empty() {
                        (conn_blocked as f64 / app_conns_summary.len() as f64) * 100.0
                    } else {
                        0.0
                    };
                    (alive, pct, r, sent)
                } else {
                    let pct = if !app_conns_summary.is_empty() {
                        (conn_blocked as f64 / app_conns_summary.len() as f64) * 100.0
                    } else {
                        0.0
                    };
                    ((conn_alive as u64).max(conn_count), pct, conn_recv, conn_sent)
                };

                let state_vpn = state.clone();
                let app_id_vpn = app_id.clone();
                let state_split = state.clone();
                let app_id_split = app_id.clone();

                let state_act = state.clone();
                let app_id_act = app_id.clone();

                let detail_id = app_id.clone();
                let detail_source = p.source.clone();
                let detail_path = path.clone();
                let detail_fp_count = p.fingerprints.len();
                let detail_is_custom = p.config.is_some();

                view! {
                    <div class="space-y-6">
                        // APPLICATION HEADER (Icon, Metadata, Quick Controls, Traffic Stats)
                        <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-5 flex flex-col md:flex-row md:items-center justify-between gap-6">
                            // Left: 3rem Monogram, Name, Path, Metadata
                            <div class="flex items-start space-x-4 min-w-0">
                                <div class="w-12 h-12 rounded bg-[#2c2c2c] border border-[#343434] flex items-center justify-center text-white font-bold text-lg flex-shrink-0">
                                    {monogram}
                                </div>

                                <div class="space-y-1 min-w-0">
                                    <div class="flex items-center space-x-2">
                                        <h2 class="text-lg font-bold text-white truncate">{display_name}</h2>
                                        {if is_vpn {
                                            view! {
                                                <span class="px-2 py-0.5 rounded bg-[#4e97fa]/20 text-[#4e97fa] text-[10px] font-mono border border-[#4e97fa]/30">
                                                    "VPN ACTIVE"
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }}
                                    </div>

                                    <div class="text-[11px] text-[#888888] font-mono truncate max-w-lg">
                                        {path}
                                    </div>

                                    // Quick Action Pills
                                    <div class="flex flex-wrap items-center gap-2 pt-2 text-xs">
                                        // Internet Access Selector
                                        <div class="inline-flex rounded bg-[#222222] border border-[#2c2c2c] p-0.5">
                                            <button
                                                on:click={
                                                    let s = state_act.clone();
                                                    let id = app_id_act.clone();
                                                    move |_| s.set_app_action(id.clone(), "permit".to_string())
                                                }
                                                class=if action == "permit" {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#143d24] text-[#1de966] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Permit"
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_act.clone();
                                                    let id = app_id_act.clone();
                                                    move |_| s.set_app_action(id.clone(), "ask".to_string())
                                                }
                                                class=if action == "ask" {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#3d3a14] text-[#e9d81d] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Ask"
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_act.clone();
                                                    let id = app_id_act.clone();
                                                    move |_| s.set_app_action(id.clone(), "block".to_string())
                                                }
                                                class=if action == "block" {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#3d1414] text-[#e01d1d] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Block"
                                            </button>
                                        </div>

                                        // 3-State VPN Routing Selector (Default / Always VPN / Direct)
                                        <div class="inline-flex rounded bg-[#222222] border border-[#2c2c2c] p-0.5">
                                            <button
                                                on:click={
                                                    let s = state_vpn.clone();
                                                    let id = app_id_vpn.clone();
                                                    move |_| s.set_app_vpn_override(id.clone(), None)
                                                }
                                                class=if vpn_override.is_none() {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#2a3950] text-[#8ab4f8] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                {if global_vpn { "VPN: Default (ON)" } else { "VPN: Default (OFF)" }}
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_vpn.clone();
                                                    let id = app_id_vpn.clone();
                                                    move |_| s.set_app_vpn_override(id.clone(), Some(true))
                                                }
                                                class=if vpn_override == Some(true) {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#16345c] text-[#4e97fa] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Always VPN"
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_vpn.clone();
                                                    let id = app_id_vpn.clone();
                                                    move |_| s.set_app_vpn_override(id.clone(), Some(false))
                                                }
                                                class=if vpn_override == Some(false) {
                                                    "px-2.5 py-1 rounded-[2px] bg-[#342a18] text-[#e9b84a] font-medium text-[11px]"
                                                } else {
                                                    "px-2.5 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Direct"
                                            </button>
                                        </div>

                                        // 3-State Split Tunnel Selector (Default / On / Off)
                                        <div class="inline-flex rounded bg-[#222222] border border-[#2c2c2c] p-0.5">
                                            <button
                                                on:click={
                                                    let s = state_split.clone();
                                                    let id = app_id_split.clone();
                                                    move |_| s.set_app_split_tun_override(id.clone(), None)
                                                }
                                                class=if split_override.is_none() {
                                                    "px-2 py-1 rounded-[2px] bg-[#2a3950] text-[#8ab4f8] font-medium text-[11px]"
                                                } else {
                                                    "px-2 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                {if global_split { "Split: Default (ON)" } else { "Split: Default (OFF)" }}
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_split.clone();
                                                    let id = app_id_split.clone();
                                                    move |_| s.set_app_split_tun_override(id.clone(), Some(true))
                                                }
                                                class=if split_override == Some(true) {
                                                    "px-2 py-1 rounded-[2px] bg-[#143d24] text-[#1de966] font-medium text-[11px]"
                                                } else {
                                                    "px-2 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Split ON"
                                            </button>
                                            <button
                                                on:click={
                                                    let s = state_split.clone();
                                                    let id = app_id_split.clone();
                                                    move |_| s.set_app_split_tun_override(id.clone(), Some(false))
                                                }
                                                class=if split_override == Some(false) {
                                                    "px-2 py-1 rounded-[2px] bg-[#333333] text-[#cacaca] font-medium text-[11px]"
                                                } else {
                                                    "px-2 py-1 rounded-[2px] text-[#888888] hover:text-white text-[11px] transition-colors"
                                                }
                                            >
                                                "Split OFF"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // 4 Stat Cards matching Portmaster header (Connections, Blocked %, Received, Sent)
                            <div class="flex flex-wrap items-center gap-3 flex-shrink-0">
                                <div class="px-3.5 py-2 rounded bg-[#222222] border border-[#2c2c2c] text-center min-w-[80px]">
                                    <div class="text-xs font-bold text-white font-mono">{alive_conns}</div>
                                    <div class="text-[10px] text-[#888888]">"Connections"</div>
                                </div>
                                <div class="px-3.5 py-2 rounded bg-[#222222] border border-[#2c2c2c] text-center min-w-[80px]">
                                    <div class="text-xs font-bold text-white font-mono">{format!("{:.0}%", blocked_pct)}</div>
                                    <div class="text-[10px] text-[#888888]">"Blocked"</div>
                                </div>
                                <div class="px-3.5 py-2 rounded bg-[#222222] border border-[#2c2c2c] text-center min-w-[90px]">
                                    <div class="text-xs font-bold text-[#1de966] font-mono">{ConnectionRecord::format_bytes(bytes_recv)}</div>
                                    <div class="text-[10px] text-[#888888]">"Received"</div>
                                </div>
                                <div class="px-3.5 py-2 rounded bg-[#222222] border border-[#2c2c2c] text-center min-w-[90px]">
                                    <div class="text-xs font-bold text-[#4e97fa] font-mono">{ConnectionRecord::format_bytes(bytes_sent)}</div>
                                    <div class="text-[10px] text-[#888888]">"Sent"</div>
                                </div>
                            </div>
                        </div>

                        // TABS NAVIGATION (Connections, Rules, Settings, Details)
                        <div class="flex items-center space-x-1 border-b border-[#2c2c2c] text-xs">
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Connections)
                                class=move || {
                                    if active_tab.get() == AppTab::Connections {
                                        "px-4 py-2 border-b-2 border-[#4e97fa] text-white font-semibold"
                                    } else {
                                        "px-4 py-2 text-[#888888] hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Connections"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Rules)
                                class=move || {
                                    if active_tab.get() == AppTab::Rules {
                                        "px-4 py-2 border-b-2 border-[#4e97fa] text-white font-semibold"
                                    } else {
                                        "px-4 py-2 text-[#888888] hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Rules & Routing"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Settings)
                                class=move || {
                                    if active_tab.get() == AppTab::Settings {
                                        "px-4 py-2 border-b-2 border-[#4e97fa] text-white font-semibold"
                                    } else {
                                        "px-4 py-2 text-[#888888] hover:text-white transition-colors"
                                    }
                                }
                            >
                                "App Settings"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Details)
                                class=move || {
                                    if active_tab.get() == AppTab::Details {
                                        "px-4 py-2 border-b-2 border-[#4e97fa] text-white font-semibold"
                                    } else {
                                        "px-4 py-2 text-[#888888] hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Details"
                            </button>
                        </div>

                        // TABS CONTENT
                        <div>
                            {move || match active_tab.get() {
                                // 1. Connections Tab
                                AppTab::Connections => {
                                    let app_id_curr = app_id.clone();
                                    let app_name_curr = display_name_match.clone();
                                    let conns = connections.get();
                                    let app_conns: Vec<_> = conns
                                        .into_iter()
                                        .filter(|c| {
                                            c.profile_id == app_id_curr
                                                || c.profile_id.ends_with(&format!("/{}", app_id_curr))
                                                || (!app_name_curr.is_empty() && c.profile_name.eq_ignore_ascii_case(&app_name_curr))
                                        })
                                        .collect();

                                    view! {
                                        <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded overflow-hidden">
                                            <div class="px-4 py-3 border-b border-[#2c2c2c] flex items-center justify-between text-xs">
                                                <span class="font-medium text-white">
                                                    {format!("Active & Recent Connections ({})", app_conns.len())}
                                                </span>
                                                <span class="text-[11px] text-[#888888]">
                                                    "Filtered specifically for this profile"
                                                </span>
                                            </div>

                                            {if app_conns.is_empty() {
                                                view! {
                                                    <div class="p-8 text-center text-xs text-[#888888]">
                                                        "No active connections recorded for this application."
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let state_act = expect_context::<AppState>();
                                                view! {
                                                    <table class="w-full text-left text-xs border-collapse">
                                                        <thead>
                                                            <tr class="bg-[#121213] border-b border-[#2c2c2c] text-[#888888] font-medium text-[11px]">
                                                                <th class="py-2.5 px-3 w-10 text-center">"Status"</th>
                                                                <th class="py-2.5 px-3">"Target"</th>
                                                                <th class="py-2.5 px-3">"Location"</th>
                                                                <th class="py-2.5 px-3">"Protocol"</th>
                                                                <th class="py-2.5 px-3 text-right">"Traffic"</th>
                                                                <th class="py-2.5 px-3 text-right">"Time"</th>
                                                                <th class="py-2.5 px-3 text-right">"Actions"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody class="divide-y divide-[#2c2c2c]/60">
                                                            {app_conns.into_iter().map(|c| {
                                                                let dot_class = if c.is_permitted() {
                                                                    "w-2.5 h-2.5 rounded-full bg-[#1de966] inline-block"
                                                                } else if c.verdict == "ask" {
                                                                    "w-2.5 h-2.5 rounded-full bg-[#e9d81d] inline-block"
                                                                } else {
                                                                    "w-2.5 h-2.5 rounded-full bg-[#e01d1d] inline-block"
                                                                };
                                                                let domain = c.domain.clone().unwrap_or_else(|| c.remote_ip.clone());
                                                                let location = c.country.clone().unwrap_or_else(|| "Local".to_string());
                                                                let id_a = c.id.clone();
                                                                let id_b = c.id.clone();
                                                                let s1 = state_act.clone();
                                                                let s2 = state_act.clone();

                                                                view! {
                                                                    <tr class="hover:bg-[#222222] transition-colors">
                                                                        <td class="py-2 px-3 text-center">
                                                                            <div class="flex items-center justify-center space-x-1">
                                                                                <span class=dot_class title=c.verdict.clone()></span>
                                                                                {if c.active {
                                                                                    view! { <span class="w-1.5 h-1.5 rounded-full bg-[#1de966] animate-pulse" title="Active"></span> }.into_any()
                                                                                } else {
                                                                                    view! { <span/> }.into_any()
                                                                                }}
                                                                            </div>
                                                                        </td>
                                                                        <td class="py-2 px-3 font-medium text-white">
                                                                            <div>{domain}</div>
                                                                            <div class="text-[10px] text-[#888888] font-mono">{format!("{}:{}", c.remote_ip, c.remote_port)}</div>
                                                                        </td>
                                                                        <td class="py-2 px-3 text-[#ababab]">
                                                                            {location}
                                                                        </td>
                                                                        <td class="py-2 px-3 font-mono text-[11px] text-[#ababab]">
                                                                            {c.protocol}
                                                                        </td>
                                                                        <td class="py-2 px-3 text-right font-mono text-[11px] whitespace-nowrap">
                                                                            <div class="text-[#1de966]">{format!("↓ {}", ConnectionRecord::format_bytes(c.bytes_received))}</div>
                                                                            <div class="text-[#4e97fa]">{format!("↑ {}", ConnectionRecord::format_bytes(c.bytes_sent))}</div>
                                                                        </td>
                                                                        <td class="py-2 px-3 text-right text-[11px] text-[#888888]">
                                                                            {c.time_ago}
                                                                        </td>
                                                                        <td class="py-2 px-3 text-right space-x-1">
                                                                            <button
                                                                                on:click=move |_| s1.allow_connection(&id_a)
                                                                                class="btn-pm !py-0.5 !px-2 !bg-[#143d24] text-[#1de966] text-[10px]"
                                                                            >
                                                                                "Permit"
                                                                            </button>
                                                                            <button
                                                                                on:click=move |_| s2.block_connection(&id_b)
                                                                                class="btn-pm !py-0.5 !px-2 !bg-[#3d1414] text-[#e01d1d] text-[10px]"
                                                                            >
                                                                                "Block"
                                                                            </button>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                }.into_any()
                                            }}
                                        </div>
                                    }.into_any()
                                }

                                // 2. Rules Tab
                                AppTab::Rules => {
                                    let cur_p = p.clone();
                                    let rules: Vec<String> = cur_p.config.as_ref()
                                        .and_then(|c| c.get("filter/endpoints"))
                                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                                        .unwrap_or_default();
                                    let app_id_add = cur_p.id.clone();

                                    view! {
                                        <div class="space-y-4">
                                            // Add New Rule Bar
                                            <div class="p-3 bg-[#1b1b1b] border border-[#2c2c2c] rounded flex flex-col sm:flex-row items-center gap-2">
                                                <input
                                                    type="text"
                                                    placeholder="Add domain, IP, or wildcard rule (e.g. *.telemetry.com)..."
                                                    prop:value=move || new_rule.get()
                                                    on:input=move |ev| set_new_rule.set(event_target_value(&ev))
                                                    class="input-pm flex-1"
                                                />
                                                <select
                                                    prop:value=move || new_rule_action.get()
                                                    on:change=move |ev| set_new_rule_action.set(event_target_value(&ev))
                                                    class="input-pm !w-auto"
                                                >
                                                    <option value="permit">"Permit"</option>
                                                    <option value="block">"Block"</option>
                                                </select>
                                                <button
                                                    on:click={
                                                        let s = state.clone();
                                                        let app_id_str = app_id_add.clone();
                                                        move |_| {
                                                            let pattern = new_rule.get().trim().to_string();
                                                            if !pattern.is_empty() {
                                                                let prefix = match new_rule_action.get().as_str() {
                                                                    "permit" => "+",
                                                                    _ => "-",
                                                                };
                                                                let rule_str = format!("{} {}", prefix, pattern);
                                                                let bare = app_id_str.rsplit('/').next().unwrap_or(&app_id_str);
                                                                let mut current_rules: Vec<String> = s
                                                                    .profiles
                                                                    .get_untracked()
                                                                    .into_iter()
                                                                    .find(|p| p.id == app_id_str || p.id == bare)
                                                                    .and_then(|p| p.config)
                                                                    .and_then(|c| c.get("filter/endpoints").cloned())
                                                                    .and_then(|v| serde_json::from_value(v).ok())
                                                                    .unwrap_or_default();
                                                                if !current_rules.contains(&rule_str) {
                                                                    current_rules.push(rule_str);
                                                                    s.save_profile_config(&app_id_str, "filter/endpoints", Some(serde_json::json!(current_rules)));
                                                                }
                                                                set_new_rule.set(String::new());
                                                            }
                                                        }
                                                    }
                                                    class="btn-pm !bg-[#4e97fa] hover:!bg-[#3d86e8] text-white font-medium"
                                                >
                                                    "Add Rule"
                                                </button>
                                            </div>

                                            // Active Rules Table
                                            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded overflow-hidden">
                                                <table class="w-full text-left text-xs">
                                                    <thead>
                                                        <tr class="bg-[#121213] border-b border-[#2c2c2c] text-[#888888] text-[11px]">
                                                            <th class="py-2.5 px-3">"Rule Pattern"</th>
                                                            <th class="py-2.5 px-3">"Action"</th>
                                                            <th class="py-2.5 px-3">"Scope"</th>
                                                            <th class="py-2.5 px-3 text-right">"Manage"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-[#2c2c2c]/60">
                                                        {if rules.is_empty() {
                                                            view! {
                                                                <tr>
                                                                    <td colspan="4" class="py-4 text-center text-[#888888]">
                                                                        "No custom rules configured for this profile."
                                                                    </td>
                                                                </tr>
                                                            }.into_any()
                                                        } else {
                                                            let app_id_del = cur_p.id.clone();
                                                            rules.into_iter().map(|rule_entry| {
                                                                let s = state.clone();
                                                                let prof_id = app_id_del.clone();
                                                                let rule_str = rule_entry.clone();
                                                                let is_permit = rule_str.starts_with('+') || rule_str.starts_with("allow") || rule_str.starts_with("permit");
                                                                let pattern = rule_str.trim_start_matches('+').trim_start_matches('-').trim_start_matches("allow").trim_start_matches("block").trim().to_string();
                                                                let action_label = if is_permit { "Permit" } else { "Block" };
                                                                let action_class = if is_permit { "py-2.5 px-3 text-[#1de966] font-semibold" } else { "py-2.5 px-3 text-[#e01d1d] font-semibold" };
                                                                let to_remove = rule_entry.clone();
                                                                let to_promote = rule_entry.clone();
                                                                let s_prom = state.clone();

                                                                view! {
                                                                    <tr class="hover:bg-[#222222] transition-colors">
                                                                        <td class="py-2.5 px-3 font-mono text-white">{pattern}</td>
                                                                        <td class=action_class>{action_label}</td>
                                                                        <td class="py-2.5 px-3 text-[#888888]">"Any"</td>
                                                                        <td class="py-2.5 px-3 text-right space-x-2">
                                                                            <button
                                                                                on:click=move |_| {
                                                                                    s_prom.promote_rule_to_global("filter/endpoints".to_string(), to_promote.clone());
                                                                                }
                                                                                class="text-[11px] text-[#4e97fa] hover:underline cursor-pointer"
                                                                            >
                                                                                {t(lang.get(), "app.promote_global")}
                                                                            </button>
                                                                            <button
                                                                                on:click={
                                                                                    let s_del = s.clone();
                                                                                    let pid = prof_id.clone();
                                                                                    let target_rem = to_remove.clone();
                                                                                    move |_| {
                                                                                        let mut remaining: Vec<String> = s_del.profiles.get().into_iter()
                                                                                            .find(|p| p.id == pid)
                                                                                            .and_then(|p| p.config)
                                                                                            .and_then(|c| c.get("filter/endpoints").cloned())
                                                                                            .and_then(|v| serde_json::from_value(v).ok())
                                                                                            .unwrap_or_default();
                                                                                        remaining.retain(|r| r != &target_rem);
                                                                                        s_del.save_profile_config(&pid, "filter/endpoints", Some(serde_json::json!(remaining)));
                                                                                    }
                                                                                }
                                                                                class="text-xs text-[#888888] hover:text-[#e01d1d] cursor-pointer"
                                                                            >
                                                                                "Delete"
                                                                            </button>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }).collect_view().into_any()
                                                        }}
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    }.into_any()
                                }

                                // 3. Settings Overrides Tab
                                AppTab::Settings => {
                                    let cur_p = p.clone();
                                    let is_lan_blocked = cur_p.config.as_ref()
                                        .and_then(|c| c.get("filter/blockLAN").or_else(|| c.get("filter/blockLan")))
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    let current_dns = cur_p.config.as_ref()
                                        .and_then(|c| c.get("dns/servers"))
                                        .and_then(|v| {
                                            if let Some(arr) = v.as_array() {
                                                let servers: Vec<String> = arr
                                                    .iter()
                                                    .filter_map(|value| value.as_str().map(String::from))
                                                    .collect();
                                                Some(servers.join(", "))
                                            } else {
                                                v.as_str().map(String::from)
                                            }
                                        })
                                        .unwrap_or_default();
                                    let app_id_cfg1 = cur_p.id.clone();
                                    let app_id_cfg2 = cur_p.id.clone();
                                    let app_id_cfg3 = cur_p.id.clone();

                                    view! {
                                        <div class="space-y-3">
                                            <div class="p-3 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded flex items-center justify-between">
                                                <div class="space-y-0.5">
                                                    <div class="text-xs font-semibold text-white">"Block LAN / Local Subnet Access"</div>
                                                    <div class="text-[11px] text-[#888888]">"Prevents application from discovering devices or communicating over local network."</div>
                                                </div>
                                                <input
                                                    type="checkbox"
                                                    prop:checked=move || is_lan_blocked
                                                    on:change={
                                                        let s = state.clone();
                                                        let pid = app_id_cfg1.clone();
                                                        move |_| {
                                                            s.save_profile_config(&pid, "filter/blockLAN", Some(serde_json::json!(!is_lan_blocked)));
                                                        }
                                                    }
                                                    class="w-4 h-4 rounded bg-[#222222] border-[#2c2c2c] text-[#4e97fa] cursor-pointer"
                                                />
                                            </div>

                                            <div class="p-3 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded space-y-2">
                                                <div class="flex items-center justify-between">
                                                    <div class="space-y-0.5">
                                                        <div class="text-xs font-semibold text-white">"Custom App DNS Upstream"</div>
                                                        <div class="text-[11px] text-[#888888]">"Override global DNS resolver specifically for this application."</div>
                                                    </div>
                                                    <button
                                                        on:click={
                                                            let s = state.clone();
                                                            let pid = app_id_cfg2.clone();
                                                            move |_| {
                                                                s.save_profile_config(&pid, "dns/servers", None);
                                                            }
                                                        }
                                                        class="btn-pm text-[10px] text-[#ababab]"
                                                    >
                                                        "Reset to Global"
                                                    </button>
                                                </div>
                                                <input
                                                    type="text"
                                                    placeholder="Inheriting Global DNS (e.g. Quad9 / Cloudflare)"
                                                    prop:value=move || current_dns.clone()
                                                    on:change={
                                                        let s = state.clone();
                                                        let pid = app_id_cfg3.clone();
                                                        move |ev| {
                                                            let servers: Vec<String> = event_target_value(&ev)
                                                                .split(',')
                                                                .map(str::trim)
                                                                .filter(|value| !value.is_empty())
                                                                .map(String::from)
                                                                .collect();
                                                            if servers.is_empty() {
                                                                s.save_profile_config(&pid, "dns/servers", None);
                                                            } else {
                                                                s.save_profile_config(&pid, "dns/servers", Some(serde_json::json!(servers)));
                                                            }
                                                        }
                                                    }
                                                    class="input-pm max-w-sm"
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                }

                                // 4. Details + Path Rule Match Tester (#2166) Tab
                                AppTab::Details => {
                                    let id_d = detail_id.clone();
                                    let src_d = detail_source.clone();
                                    let path_d = detail_path.clone();
                                    let fps_for_test = p.fingerprints.clone();
                                    let linked_for_test = p.linked_path.clone();
                                    let pres_for_test = p.presentation_path.clone();

                                    view! {
                                        <div class="space-y-4 text-xs">
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                                <div class="p-4 bg-[#1b1b1b] border border-[#2c2c2c] rounded space-y-2">
                                                    <div class="text-[11px] text-[#888888] font-medium uppercase">"Profile Identity"</div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"App ID:"</span>
                                                        <span class="font-mono text-white">{id_d}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"Source:"</span>
                                                        <span class="font-mono text-white">{src_d}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"Binary Path:"</span>
                                                        <span class="font-mono text-[#ababab] break-all">{path_d}</span>
                                                    </div>
                                                </div>

                                                <div class="p-4 bg-[#1b1b1b] border border-[#2c2c2c] rounded space-y-2">
                                                    <div class="text-[11px] text-[#888888] font-medium uppercase">"Fingerprints & Status"</div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"Fingerprints:"</span>
                                                        <span class="text-white">{detail_fp_count}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"Config Overrides:"</span>
                                                        <span class="text-white">{if detail_is_custom { "Customized" } else { "Global Defaults" }}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-[#888888] mr-2">"VPN Tunneling:"</span>
                                                        <span class="text-white">{if is_vpn { "Enabled (Routed)" } else { "Disabled (Direct)" }}</span>
                                                    </div>
                                                </div>
                                            </div>

                                            // Path Fingerprint Rule Match Tester (#2166)
                                            <div class="p-4 bg-[#1b1b1b] border border-[#2c2c2c] rounded space-y-2.5">
                                                <div class="text-xs font-semibold text-white">{t(lang.get(), "app.path_tester")}</div>
                                                <p class="text-[11px] text-[#888888]">{t(lang.get(), "app.path_tester_hint")}</p>
                                                <input
                                                    type="text"
                                                    placeholder=r"/usr/bin/firefox or C:\Program Files\..."
                                                    prop:value=move || test_path.get()
                                                    on:input=move |ev| set_test_path.set(event_target_value(&ev))
                                                    class="input-pm text-xs max-w-xl"
                                                />
                                                {move || {
                                                    let input = test_path.get();
                                                    let trimmed = input.trim();
                                                    if trimmed.is_empty() {
                                                        return view! { <div class="text-[11px] text-[#666666]">"Enter a binary path to test."</div> }.into_any();
                                                    }
                                                    let norm = trimmed.replace('\\', "/").to_lowercase();
                                                    let linked_norm = linked_for_test.replace('\\', "/").to_lowercase();
                                                    let pres_norm = pres_for_test.replace('\\', "/").to_lowercase();
                                                    let mut matched_reason: Option<String> = None;
                                                    if !linked_norm.is_empty() && linked_norm == norm {
                                                        matched_reason = Some(format!("Exact LinkedPath: {}", linked_for_test));
                                                    } else if !pres_norm.is_empty() && pres_norm == norm {
                                                        matched_reason = Some(format!("Exact PresentationPath: {}", pres_for_test));
                                                    } else {
                                                        for fp in &fps_for_test {
                                                            if fp.fp_type == FingerprintType::Path {
                                                                let v = fp.value.replace('\\', "/").to_lowercase();
                                                                let hit = match fp.operation {
                                                                    FingerprintOperation::Equal => norm == v,
                                                                    FingerprintOperation::Prefix => norm.starts_with(&v),
                                                                    FingerprintOperation::Regex => !v.is_empty() && norm.contains(v.trim_matches(['^', '$', '*'])),
                                                                    FingerprintOperation::Unknown => norm == v,
                                                                };
                                                                if hit {
                                                                    matched_reason = Some(format!("Matched Fingerprint ({:?}): {}", fp.operation, fp.value));
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    if let Some(reason) = matched_reason {
                                                        view! {
                                                            <div class="p-2 rounded bg-[#143d24] border border-[#1de966]/40 text-[#1de966] font-mono text-[11px]">
                                                                {format!("✓ MATCH — {}", reason)}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="p-2 rounded bg-[#3d1414] border border-[#e01d1d]/30 text-[#e01d1d] font-mono text-[11px]">
                                                                "✗ NO MATCH — Does not match this profile's path/fingerprints."
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                },
                            }}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
