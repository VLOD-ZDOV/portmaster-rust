//! Application list & routing controls styled 1:1 with Portmaster App View.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn AppSwitcher() -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let filter_query = state.filter_query;
    let _set_filter_query = state.set_filter_query;
    let route_by_default = state.route_by_default;

    // Filter tab: "all", "vpn", "direct", "blocked"
    let (active_tab, set_active_tab) = signal("all".to_string());

    view! {
        <div class="space-y-4">
            // Filter Bar matching Portmaster's App View Sub-header
            <div class="flex items-center justify-between pb-3 border-b border-[#2c2c2c] flex-wrap gap-3">
                <div class="flex items-center space-x-3">
                    <span class="text-sm font-semibold text-white">
                        {move || format!("{} Applications", profiles.get().len())}
                    </span>
                    <span class="text-xs text-[#666666]">
                        {move || {
                            let def = route_by_default.get();
                            if def {
                                "Global VPN Default: ON"
                            } else {
                                "Global VPN Default: OFF (Direct)"
                            }
                        }}
                    </span>
                </div>

                // Tabs: All / VPN / Direct / Blocked
                <div class="flex items-center space-x-1 bg-[#1b1b1b] p-1 rounded-md border border-[#2c2c2c] text-xs">
                    <button
                        on:click=move |_| set_active_tab.set("all".to_string())
                        class=move || {
                            if active_tab.get() == "all" {
                                "px-3 py-1 rounded bg-[#2c2c2c] text-white font-medium shadow-sm"
                            } else {
                                "px-3 py-1 rounded text-[#ababab] hover:text-white transition-colors"
                            }
                        }
                    >
                        "All"
                    </button>
                    <button
                        on:click=move |_| set_active_tab.set("vpn".to_string())
                        class=move || {
                            if active_tab.get() == "vpn" {
                                "px-3 py-1 rounded bg-[#4e97fa]/20 text-[#4e97fa] font-medium border border-[#4e97fa]/30"
                            } else {
                                "px-3 py-1 rounded text-[#ababab] hover:text-white transition-colors"
                            }
                        }
                    >
                        "VPN Routed"
                    </button>
                    <button
                        on:click=move |_| set_active_tab.set("direct".to_string())
                        class=move || {
                            if active_tab.get() == "direct" {
                                "px-3 py-1 rounded bg-[#2c2c2c] text-white font-medium shadow-sm"
                            } else {
                                "px-3 py-1 rounded text-[#ababab] hover:text-white transition-colors"
                            }
                        }
                    >
                        "Direct"
                    </button>
                    <button
                        on:click=move |_| set_active_tab.set("blocked".to_string())
                        class=move || {
                            if active_tab.get() == "blocked" {
                                "px-3 py-1 rounded bg-[#e01d1d]/20 text-[#e01d1d] font-medium border border-[#e01d1d]/30"
                            } else {
                                "px-3 py-1 rounded text-[#ababab] hover:text-white transition-colors"
                            }
                        }
                    >
                        "Blocked"
                    </button>
                </div>
            </div>

            // App list rows
            <div class="space-y-2">
                {move || {
                    let q = filter_query.get().to_lowercase();
                    let tab = active_tab.get();
                    let _def_route = route_by_default.get();
                    let all = profiles.get();

                    let filtered: Vec<_> = all
                        .into_iter()
                        .filter(|p| {
                            // Text filter
                            let matches_query = q.is_empty()
                                || p.name.to_lowercase().contains(&q)
                                || p.presentation_path.to_lowercase().contains(&q)
                                || p.id.to_lowercase().contains(&q);

                            if !matches_query {
                                return false;
                            }

                            // Tab filter
                            let is_vpn = p.route_via_vpn();
                            let is_blocked = p.default_action() == "block";

                            match tab.as_str() {
                                "vpn" => is_vpn,
                                "direct" => !is_vpn,
                                "blocked" => is_blocked,
                                _ => true,
                            }
                        })
                        .collect();

                    if filtered.is_empty() {
                        view! {
                            <div class="py-12 text-center text-xs text-[#666666] bg-[#1b1b1b] rounded-lg border border-[#2c2c2c]">
                                "No applications match the selected filter."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-2">
                                {filtered.into_iter().map(|app| {
                                    let app_id_vpn = app.id.clone();
                                    let app_id_block = app.id.clone();
                                    let is_vpn = app.route_via_vpn();
                                    let action = app.default_action().to_string();
                                    let is_blocked = action == "block";

                                    let initials = app.name.chars().take(2).collect::<String>().to_uppercase();

                                    let state_clone = state.clone();
                                    let on_toggle_vpn = move |_| {
                                        state_clone.toggle_app_vpn(app_id_vpn.clone());
                                    };

                                    let state_clone2 = state.clone();
                                    let next_action = if is_blocked { "permit".to_string() } else { "block".to_string() };
                                    let on_toggle_block = move |_| {
                                        state_clone2.set_app_action(app_id_block.clone(), next_action.clone());
                                    };

                                    view! {
                                        <div class="bg-[#1b1b1b] border border-[#2c2c2c] hover:border-[#3a3a3a] rounded-lg p-3 flex items-center justify-between gap-4 transition-all">
                                            // Left: Monogram icon & App Info
                                            <div class="flex items-center space-x-3 min-w-0">
                                                <div class="w-10 h-10 rounded bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-white font-semibold text-xs flex-shrink-0">
                                                    {initials}
                                                </div>
                                                <div class="min-w-0">
                                                    <div class="flex items-center space-x-2">
                                                        <span class="text-sm font-medium text-white truncate">{app.name.clone()}</span>
                                                        {if is_blocked {
                                                            view! {
                                                                <span class="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-[#e01d1d]/20 text-[#e01d1d] border border-[#e01d1d]/30">
                                                                    "Blocked"
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span/> }.into_any()
                                                        }}
                                                    </div>
                                                    <p class="text-xs text-[#666666] font-mono truncate">{app.presentation_path.clone()}</p>
                                                </div>
                                            </div>

                                            // Right: Quick Action Controls
                                            <div class="flex items-center space-x-2 flex-shrink-0">
                                                // VPN Routing Toggle Button
                                                <button
                                                    on:click=on_toggle_vpn
                                                    title=if is_vpn { "Routing traffic via VPN (Xray)" } else { "Bypassing VPN (Direct connection)" }
                                                    class=if is_vpn {
                                                        "px-3 py-1.5 rounded text-xs font-medium bg-[#4e97fa]/15 text-[#4e97fa] border border-[#4e97fa]/40 hover:bg-[#4e97fa]/25 flex items-center space-x-1.5 transition-colors"
                                                    } else {
                                                        "px-3 py-1.5 rounded text-xs font-medium bg-[#222222] text-[#ababab] border border-[#2c2c2c] hover:border-[#3a3a3a] hover:text-white flex items-center space-x-1.5 transition-colors"
                                                    }
                                                >
                                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                        <circle cx="12" cy="12" r="9" />
                                                        <path d="M3.6 9h16.8M3.6 15h16.8" />
                                                    </svg>
                                                    <span>{if is_vpn { "VPN: ON" } else { "VPN: OFF" }}</span>
                                                </button>

                                                // Block/Permit Internet Toggle
                                                <button
                                                    on:click=on_toggle_block
                                                    title=if is_blocked { "Permit Internet" } else { "Block Internet" }
                                                    class=if is_blocked {
                                                        "px-3 py-1.5 rounded text-xs font-medium bg-[#e01d1d]/20 text-[#e01d1d] border border-[#e01d1d]/40 hover:bg-[#e01d1d]/30 flex items-center space-x-1 transition-colors"
                                                    } else {
                                                        "px-3 py-1.5 rounded text-xs font-medium bg-[#222222] text-[#ababab] border border-[#2c2c2c] hover:border-[#3a3a3a] hover:text-white flex items-center space-x-1 transition-colors"
                                                    }
                                                >
                                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                                                    </svg>
                                                    <span>{if is_blocked { "Unblock" } else { "Block" }}</span>
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
