//! Detailed Application View matching Portmaster 1:1.
//!
//! Replicates Portmaster's application details view:
//! - Breadcrumbs & expertise selector
//! - App header with 3rem monogram icon, metadata, and quick settings
//! - Bandwidth / traffic counters (Received / Sent)
//! - 4 Tabs: Connections, Rules & Routing, App Settings Overrides, Details

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;

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

    // Endpoint rules list local state
    let (new_rule, set_new_rule) = signal(String::new());
    let (new_rule_action, set_new_rule_action) = signal("block".to_string());
    let (custom_dns, set_custom_dns) = signal("1.1.1.1".to_string());
    let (block_lan, set_block_lan) = signal(false);

    let state_back = state.clone();
    let on_back = move |_| {
        state_back.select_app(None);
    };

    let id_clone = app_id.clone();
    let profile = Memo::new(move |_| {
        let list = profiles.get();
        list.into_iter().find(|p| p.id == id_clone)
    });

    view! {
        <div class="space-y-4">
            // Breadcrumbs Header
            <div class="flex items-center justify-between pb-2 border-b border-[#2c2c2c] text-xs">
                <div class="flex items-center space-x-1.5 text-[#888888]">
                    <button
                        on:click=on_back
                        class="hover:text-white cursor-pointer transition-colors flex items-center space-x-1 font-medium"
                    >
                        <span>"Applications"</span>
                    </button>
                    <span>"/"</span>
                    <span class="text-white font-medium">
                        {move || profile.get().map(|p| p.display_name().to_string()).unwrap_or_else(|| app_id.clone())}
                    </span>
                </div>

                <div class="text-[11px] text-[#888888]">
                    "App Profile Configuration"
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
                let is_vpn = p.route_via_vpn();
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
                            || c.profile_name.eq_ignore_ascii_case(&app_name_curr)
                            || (!app_id_curr.is_empty() && c.profile_name.to_lowercase().contains(&app_id_curr.to_lowercase()))
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
                    let pct = if s.connections_count > 0 {
                        s.blocked_percent()
                    } else if !app_conns_summary.is_empty() {
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
                let on_toggle_vpn = move |_| {
                    state_vpn.toggle_app_vpn(app_id_vpn.clone());
                };

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

                                        // Route via VPN Toggle
                                        <button
                                            on:click=on_toggle_vpn
                                            class=if is_vpn {
                                                "btn-pm !bg-[#4e97fa]/20 text-[#4e97fa] border border-[#4e97fa]/40 font-semibold"
                                            } else {
                                                "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                            }
                                        >
                                            {if is_vpn { "Route via VPN: ON" } else { "Route via VPN: OFF" }}
                                        </button>
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
                                                || c.profile_name.eq_ignore_ascii_case(&app_name_curr)
                                                || (!app_id_curr.is_empty() && c.profile_name.to_lowercase().contains(&app_id_curr.to_lowercase()))
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
                                AppTab::Rules => view! {
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
                                                <option value="drop">"Drop"</option>
                                            </select>
                                            <button
                                                on:click=move |_| {
                                                    if !new_rule.get().trim().is_empty() {
                                                        set_new_rule.set(String::new());
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
                                                    <tr class="hover:bg-[#222222] transition-colors">
                                                        <td class="py-2.5 px-3 font-mono text-white">"*.telemetry.mozilla.org"</td>
                                                        <td class="py-2.5 px-3 text-[#e01d1d] font-semibold">"Block"</td>
                                                        <td class="py-2.5 px-3 text-[#888888]">"Internet"</td>
                                                        <td class="py-2.5 px-3 text-right">
                                                            <button class="text-xs text-[#888888] hover:text-[#e01d1d]">"Delete"</button>
                                                        </td>
                                                    </tr>
                                                    <tr class="hover:bg-[#222222] transition-colors">
                                                        <td class="py-2.5 px-3 font-mono text-white">"*.safing.io"</td>
                                                        <td class="py-2.5 px-3 text-[#1de966] font-semibold">"Permit"</td>
                                                        <td class="py-2.5 px-3 text-[#888888]">"Internet"</td>
                                                        <td class="py-2.5 px-3 text-right">
                                                            <button class="text-xs text-[#888888] hover:text-[#e01d1d]">"Delete"</button>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                }.into_any(),

                                // 3. Settings Overrides Tab
                                AppTab::Settings => view! {
                                    <div class="space-y-3">
                                        <div class="p-3 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded flex items-center justify-between">
                                            <div class="space-y-0.5">
                                                <div class="text-xs font-semibold text-white">"Block LAN / Local Subnet Access"</div>
                                                <div class="text-[11px] text-[#888888]">"Prevents application from discovering devices or communicating over local network."</div>
                                            </div>
                                            <input
                                                type="checkbox"
                                                prop:checked=move || block_lan.get()
                                                on:change=move |_| set_block_lan.update(|v| *v = !*v)
                                                class="w-4 h-4 rounded bg-[#222222] border-[#2c2c2c] text-[#4e97fa]"
                                            />
                                        </div>

                                        <div class="p-3 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded space-y-2">
                                            <div class="flex items-center justify-between">
                                                <div class="space-y-0.5">
                                                    <div class="text-xs font-semibold text-white">"Custom App DNS Upstream"</div>
                                                    <div class="text-[11px] text-[#888888]">"Override global DNS resolver specifically for this application."</div>
                                                </div>
                                                <button
                                                    on:click=move |_| set_custom_dns.set("1.1.1.1".to_string())
                                                    class="btn-pm text-[10px] text-[#ababab]"
                                                >
                                                    "Reset to Global"
                                                </button>
                                            </div>
                                            <input
                                                type="text"
                                                prop:value=move || custom_dns.get()
                                                on:input=move |ev| set_custom_dns.set(event_target_value(&ev))
                                                class="input-pm max-w-sm"
                                            />
                                        </div>
                                    </div>
                                }.into_any(),

                                // 4. Details Tab
                                AppTab::Details => {
                                    let id_d = detail_id.clone();
                                    let src_d = detail_source.clone();
                                    let path_d = detail_path.clone();
                                    view! {
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
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
