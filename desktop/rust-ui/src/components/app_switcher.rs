//! Application list & routing controls styled 1:1 with Portmaster App Overview.
//!
//! Replicates Portmaster's overview page:
//! - Search input and manage profile menu trigger
//! - Active applications & All applications groupings
//! - Card headers in `#202020e0` with monogram icons, presentation path, blue customizer dot
//! - Quick firewall action toggle (Permit / Ask / Block) and VPN routing toggle
//! - Seamless navigation to detailed AppDetailView

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;

use crate::state::AppState;

#[component]
pub fn AppSwitcher() -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let filter_query = state.filter_query;
    let set_filter_query = state.set_filter_query;
    let route_by_default = state.route_by_default;

    // Filter tab: "all", "active", "vpn", "blocked"
    let (active_tab, set_active_tab) = signal("all".to_string());
    let (manage_menu_open, set_manage_menu_open) = signal(false);

    view! {
        <div class="space-y-4">
            // TOP BAR: Search and Breadcrumb header matching Portmaster
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-[#2c2c2c]">
                <div class="flex items-center space-x-3">
                    <div class="relative max-w-xs w-full sm:w-64">
                        <input
                            type="text"
                            placeholder="Search applications..."
                            prop:value=move || filter_query.get()
                            on:input=move |ev| set_filter_query.set(event_target_value(&ev))
                            class="input-pm !pl-8 text-xs"
                        />
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                            class="absolute w-3.5 h-3.5 top-2.5 left-2.5 text-[#888888]"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </div>

                    // Quick Filter Pills
                    <div class="flex items-center space-x-1 bg-[#1b1b1b] p-1 rounded border border-[#2c2c2c] text-xs">
                        <button
                            on:click=move |_| set_active_tab.set("all".to_string())
                            class=move || {
                                if active_tab.get() == "all" {
                                    "px-2.5 py-0.5 rounded bg-[#2c2c2c] text-white font-medium text-[11px]"
                                } else {
                                    "px-2.5 py-0.5 rounded text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "All"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("active".to_string())
                            class=move || {
                                if active_tab.get() == "active" {
                                    "px-2.5 py-0.5 rounded bg-[#2c2c2c] text-white font-medium text-[11px]"
                                } else {
                                    "px-2.5 py-0.5 rounded text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "Active"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("vpn".to_string())
                            class=move || {
                                if active_tab.get() == "vpn" {
                                    "px-2.5 py-0.5 rounded bg-[#4e97fa]/20 text-[#4e97fa] font-medium text-[11px] border border-[#4e97fa]/30"
                                } else {
                                    "px-2.5 py-0.5 rounded text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "VPN"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("blocked".to_string())
                            class=move || {
                                if active_tab.get() == "blocked" {
                                    "px-2.5 py-0.5 rounded bg-[#e01d1d]/20 text-[#e01d1d] font-medium text-[11px] border border-[#e01d1d]/30"
                                } else {
                                    "px-2.5 py-0.5 rounded text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "Blocked"
                        </button>
                    </div>
                </div>

                // Right: Manage Menu & Global VPN default toggle
                <div class="flex items-center space-x-2 relative">
                    <button
                        on:click={
                            let state = state.clone();
                            move |_| state.toggle_route_by_default()
                        }
                        class=move || {
                            let def = route_by_default.get();
                            if def {
                                "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40 flex items-center space-x-1.5"
                            } else {
                                "btn-pm !bg-[#222222] text-[#888888] hover:text-white flex items-center space-x-1.5"
                            }
                        }
                    >
                        <span>"Global VPN:"</span>
                        <span class="font-bold">
                            {move || if route_by_default.get() { "ON" } else { "OFF" }}
                        </span>
                    </button>

                    <div class="relative">
                        <button
                            on:click=move |_| set_manage_menu_open.update(|v| *v = !*v)
                            class="btn-pm !bg-[#222222] hover:!bg-[#2c2c2c] text-[#ababab] hover:text-white flex items-center space-x-1"
                        >
                            <span>"Manage"</span>
                            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>

                        {move || {
                            if manage_menu_open.get() {
                                view! {
                                    <div class="absolute right-0 mt-1 w-44 bg-[#1b1b1b] border border-[#2c2c2c] rounded shadow-xl py-1 z-50 text-xs">
                                        <button on:click=move |_| set_manage_menu_open.set(false) class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Create Profile"
                                        </button>
                                        <button on:click=move |_| set_manage_menu_open.set(false) class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Import Profile"
                                        </button>
                                        <button on:click=move |_| set_manage_menu_open.set(false) class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Merge Profiles"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>

            // APPLICATIONS LIST
            <div class="space-y-1.5">
                {move || {
                    let query = filter_query.get().to_lowercase();
                    let tab = active_tab.get();
                    let list = profiles.get();
                    let conns = state.connections.get();

                    let filtered: Vec<_> = list
                        .into_iter()
                        .filter(|p| {
                            // Search filter
                            if query.is_empty() {
                                true
                            } else {
                                p.name.to_lowercase().contains(&query)
                                    || p.id.to_lowercase().contains(&query)
                                    || p.source.to_lowercase().contains(&query)
                            }
                        })
                        .filter(|p| {
                            // Tab filter
                            match tab.as_str() {
                                "active" => {
                                    let has_live = conns.iter().any(|c| {
                                        c.active && (
                                            c.profile_id == p.id
                                                || c.profile_id.ends_with(&format!("/{}", p.id))
                                                || (!p.name.is_empty() && c.profile_name.eq_ignore_ascii_case(&p.name))
                                        )
                                    });
                                    p.count_alive_connections() > 0 || has_live
                                }
                                "vpn" => p.route_via_vpn(),
                                "blocked" => p.default_action() == "block"
                                    || p.stats.map(|s| s.blocked_count > 0).unwrap_or(false)
                                    || conns.iter().any(|c| c.is_blocked() && (c.profile_id == p.id || c.profile_id.ends_with(&format!("/{}", p.id)))),
                                _ => true,
                            }
                        })
                        .collect();

                    if filtered.is_empty() {
                        return view! {
                            <div class="p-8 text-center bg-[#1b1b1b] border border-[#2c2c2c] rounded text-[#888888] text-xs">
                                "No applications found matching the selected filter."
                            </div>
                        }.into_any();
                    }

                    let state = state.clone();
                    filtered
                        .into_iter()
                        .map(|p| {
                            let app_id = p.id.clone();
                            let app_name = p.display_name().to_string();
                            let initials = p.name.chars().take(2).collect::<String>().to_uppercase();
                            let path_desc = if !p.presentation_path.is_empty() { p.presentation_path.clone() } else { p.source.clone() };
                            let is_vpn = p.route_via_vpn();
                            let default_action = p.default_action();
                            let is_blocked = default_action == "block";

                            let (live_recv, live_sent, live_active) = {
                                let mut r = 0u64;
                                let mut s = 0u64;
                                let mut a = 0usize;
                                for c in &conns {
                                    if c.profile_id == p.id
                                        || c.profile_id.ends_with(&format!("/{}", p.id))
                                        || c.profile_name.eq_ignore_ascii_case(&p.name)
                                        || (!p.id.is_empty() && c.profile_name.to_lowercase().contains(&p.id.to_lowercase()))
                                    {
                                        r = r.saturating_add(c.bytes_received);
                                        s = s.saturating_add(c.bytes_sent);
                                        if c.active {
                                            a += 1;
                                        }
                                    }
                                }
                                (r, s, a)
                            };
                            let conn_count = p.count_alive_connections().max(live_active as u64);
                            let (bytes_recv, bytes_sent) = if let Some(s) = p.stats {
                                (s.bytes_received.max(live_recv), s.bytes_sent.max(live_sent))
                            } else {
                                (live_recv, live_sent)
                            };
                            let has_config_changes = p.config.is_some();

                            let state_vpn = state.clone();
                            let app_id_vpn = app_id.clone();
                            let on_toggle_vpn = move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                state_vpn.toggle_app_vpn(app_id_vpn.clone());
                            };

                            let state_action = state.clone();
                            let app_id_act = app_id.clone();
                            let next_action = if is_blocked { "permit".to_string() } else { "block".to_string() };
                            let on_toggle_block = move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                state_action.set_app_action(app_id_act.clone(), next_action.clone());
                            };

                            let state_nav = state.clone();
                            let app_id_nav = app_id.clone();
                            let on_open_detail = move |_| {
                                state_nav.select_app(Some(app_id_nav.clone()));
                            };

                            view! {
                                <div
                                    on:click=on_open_detail
                                    class="relative card-header-pm bg-[#202020e0] hover:bg-[#292929b0] rounded-[4px] px-3 py-2.5 flex items-center justify-between gap-3 cursor-pointer border border-transparent hover:border-[#2c2c2c] transition-all group"
                                >
                                    // Left: Monogram Icon & Name & Path
                                    <div class="flex items-center space-x-3 min-w-0">
                                        <div class="w-8 h-8 rounded bg-[#2c2c2c] flex items-center justify-center text-white font-semibold text-xs flex-shrink-0 group-hover:bg-[#343434] transition-colors">
                                            {initials}
                                        </div>

                                        <div class="min-w-0">
                                            <div class="text-xs font-semibold text-[#cacaca] group-hover:text-white truncate">
                                                {app_name}
                                            </div>
                                            <div class="text-[10px] text-[#888888] truncate font-mono">
                                                {path_desc}
                                            </div>
                                        </div>
                                    </div>

                                    // Right: Live Connection Counter & Quick Controls
                                    <div class="flex items-center space-x-2 flex-shrink-0">
                                        // Active connections badge
                                        {if conn_count > 0 {
                                            view! {
                                                <span class="px-2 py-0.5 rounded bg-[#2c2c2c] text-[#1de966] text-[10px] font-mono font-medium">
                                                    {format!("{} conn", conn_count)}
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }}

                                        // Transferred bytes badge (↓ recv, ↑ sent)
                                        {if bytes_recv > 0 || bytes_sent > 0 {
                                            view! {
                                                <span class="px-2 py-0.5 rounded bg-[#222222] border border-[#2c2c2c] text-[10px] font-mono font-medium flex items-center space-x-1.5">
                                                    <span class="text-[#1de966]">{format!("↓ {}", ConnectionRecord::format_bytes(bytes_recv))}</span>
                                                    <span class="text-[#4e97fa]">{format!("↑ {}", ConnectionRecord::format_bytes(bytes_sent))}</span>
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }}

                                        // VPN Tunnel Toggle Button
                                        <button
                                            on:click=on_toggle_vpn
                                            title="Route app traffic through VPN tunnel"
                                            class=if is_vpn {
                                                "btn-pm !bg-[#4e97fa]/20 text-[#4e97fa] border border-[#4e97fa]/40 font-semibold"
                                            } else {
                                                "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                            }
                                        >
                                            {if is_vpn { "VPN ON" } else { "VPN OFF" }}
                                        </button>

                                        // Firewall Permit / Block Switch
                                        <button
                                            on:click=on_toggle_block
                                            title="Toggle default network action"
                                            class=if is_blocked {
                                                "btn-pm !bg-[#3d1414] text-[#e01d1d] border border-[#e01d1d]/40 font-semibold"
                                            } else {
                                                "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40"
                                            }
                                        >
                                            {if is_blocked { "Blocked" } else { "Permitted" }}
                                        </button>

                                        // Blue indicator dot if custom settings applied
                                        {if has_config_changes {
                                            view! {
                                                <span title="App has customized settings" class="w-1.5 h-1.5 rounded-full bg-[#4e97fa]"></span>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }}
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}
