//! Slide-out SideDash drawer matching Portmaster 1:1.
//!
//! Includes:
//! - Security Lock with concentric animated shields & checkmark
//! - Feature Scout subsystem health badges (Secure DNS, Privacy Filter, Xray VPN, History)
//! - Network Scout live application list with search, sorting, and connection counters
//! - Notifications drawer with level badges and dismissal

use leptos::prelude::*;

use crate::i18n::t;
use crate::state::AppState;

#[component]
pub fn SideDash() -> impl IntoView {
    let state = expect_context::<AppState>();
    let is_open = state.is_side_dash_open;
    let profiles = state.profiles;
    let notifications = state.notifications;
    let xray_status = state.xray_status;
    let security_level = state.security_level;
    let conn_state = state.client.state;
    let is_paused = state.is_paused;
    let vpn_paused = state.vpn_paused;
    let settings = state.settings;
    let is_connected = move || conn_state.get() == crate::client::ConnectionState::Connected;
    let protection_paused = move || is_paused.get() && !vpn_paused.get();
    let history_enabled = move || {
        settings
            .get()
            .iter()
            .find(|setting| setting.key == "history/enable")
            .map(|setting| setting.effective_value().as_bool().unwrap_or(true))
            .unwrap_or(true)
    };

    let (scout_search, set_scout_search) = signal(String::new());
    let (sort_by, set_sort_by) = signal("Activity".to_string());

    view! {
        <aside
            class=move || {
                if is_open.get() {
                    "fixed top-0 bottom-0 left-16 w-80 bg-[#121213] border-r border-[#2c2c2c] z-30 flex flex-col p-3 space-y-4 overflow-y-auto transition-transform duration-200 translate-x-0 shadow-2xl"
                } else {
                    "fixed top-0 bottom-0 left-16 w-80 bg-[#121213] border-r border-[#2c2c2c] z-30 flex flex-col p-3 space-y-4 overflow-y-auto transition-transform duration-200 -translate-x-full shadow-none pointer-events-none"
                }
            }
        >
            // 1. SECURITY LOCK (Concentric Shields)
            <div class="flex flex-col items-center justify-center p-3 rounded bg-[#1b1b1b] border border-[#2c2c2c] relative">
                <div class="relative w-20 h-20 flex items-center justify-center mb-1">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        class="w-16 h-16 transition-all duration-200"
                    >
                        // Concentric Outer Shield (Scale 1.0)
                        <path
                            class="shield-three"
                            stroke="#143d24"
                            fill="#143d24"
                            stroke-width="1.4"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M20 11.242c0 4.368-3.157 8.462-7.48 9.686-.338.096-.702.096-1.04 0C7.157 19.705 4 15.61 4 11.242V7.214c0-.812.491-1.544 1.243-1.851l4.864-1.99c1.214-.497 2.574-.497 3.787 0l4.864 1.99C19.509 5.67 20 6.402 20 7.214v4.028z"
                        />
                        // Concentric Middle Shield (Scale 0.8)
                        <path
                            class="shield-two"
                            stroke="#18823d"
                            fill="#18823d"
                            stroke-width="1.4"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M20 11.242c0 4.368-3.157 8.462-7.48 9.686-.338.096-.702.096-1.04 0C7.157 19.705 4 15.61 4 11.242V7.214c0-.812.491-1.544 1.243-1.851l4.864-1.99c1.214-.497 2.574-.497 3.787 0l4.864 1.99C19.509 5.67 20 6.402 20 7.214v4.028z"
                        />
                        // Concentric Inner Shield (Scale 0.62)
                        <path
                            class="shield-one"
                            stroke="#1de966"
                            fill="#1de966"
                            stroke-width="1.4"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M20 11.242c0 4.368-3.157 8.462-7.48 9.686-.338.096-.702.096-1.04 0C7.157 19.705 4 15.61 4 11.242V7.214c0-.812.491-1.544 1.243-1.851l4.864-1.99c1.214-.497 2.574-.497 3.787 0l4.864 1.99C19.509 5.67 20 6.402 20 7.214v4.028z"
                        />
                        // Checkmark
                        <path
                            stroke="#ffffff"
                            fill="none"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M8.712 12.566l2.193 2.193 4.787-4.788"
                        />
                    </svg>
                </div>

                <div class="text-center space-y-0.5">
                    <h2 class="text-sm font-semibold text-white">
                        {move || if !is_connected() { "Not connected" } else if protection_paused() { "Paused" } else { "Protected" }}
                    </h2>
                    <p class="text-[11px] text-[#ababab]">
                        {move || format!("Profile: {}", security_level.get())}
                    </p>
                </div>

                <div class="flex items-center space-x-1 mt-3 w-full">
                    <button
                        on:click={
                            let state = state.clone();
                            move |_| state.update_security_level("Untrusted")
                        }
                        class=move || {
                            if security_level.get() == "Untrusted" {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#343434] text-white font-medium border border-[#4e97fa]"
                            } else {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#222222] text-[#888888] hover:text-white transition-colors"
                            }
                        }
                    >
                        "Untrusted"
                    </button>
                    <button
                        on:click={
                            let state = state.clone();
                            move |_| state.update_security_level("Standard (Balanced)")
                        }
                        class=move || {
                            if security_level.get() == "Standard (Balanced)" {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#343434] text-[#1de966] font-medium border border-[#1de966]/40"
                            } else {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#222222] text-[#888888] hover:text-white transition-colors"
                            }
                        }
                    >
                        "Standard"
                    </button>
                    <button
                        on:click={
                            let state = state.clone();
                            move |_| state.update_security_level("Strict (Block All)")
                        }
                        class=move || {
                            if security_level.get() == "Strict (Block All)" {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#343434] text-[#e01d1d] font-medium border border-[#e01d1d]/40"
                            } else {
                                "flex-1 py-1 text-[10px] rounded-[2px] bg-[#222222] text-[#888888] hover:text-white transition-colors"
                            }
                        }
                    >
                        "Strict"
                    </button>
                </div>
            </div>

            // 2. FEATURE SCOUT (Subsystem Health)
            <div class="space-y-1.5">
                <div class="flex items-center justify-between text-[11px] text-[#888888] px-0.5">
                    <span class="font-medium uppercase tracking-wider text-[10px]">"Feature Scout"</span>
                    <span>
                        {move || {
                            let mut count = 0;
                            if is_connected() && !protection_paused() {
                                count += 2;
                            }
                            if xray_status.get().connected {
                                count += 1;
                            }
                            if is_connected() && !protection_paused() && history_enabled() {
                                count += 1;
                            }
                            format!("{}/4 Active", count)
                        }}
                    </span>
                </div>

                <div class="grid grid-cols-2 gap-1.5 text-xs">
                    // Secure DNS
                    <div class="p-2 rounded bg-[#1b1b1b] border border-[#2c2c2c] flex items-center space-x-2">
                        <span class=move || {
                            if !is_connected() {
                                "w-2 h-2 rounded-full bg-[#e01d1d]"
                            } else if protection_paused() {
                                "w-2 h-2 rounded-full bg-[#e9d81d]"
                            } else {
                                "w-2 h-2 rounded-full bg-[#1de966]"
                            }
                        }></span>
                        <div class="min-w-0">
                            <div class="text-[11px] font-medium text-white truncate">"Secure DNS"</div>
                            <div class="text-[9px] text-[#888888]">
                                {move || if !is_connected() { "Offline" } else if protection_paused() { "Paused" } else { "Active" }}
                            </div>
                        </div>
                    </div>

                    // Privacy Filter
                    <div class="p-2 rounded bg-[#1b1b1b] border border-[#2c2c2c] flex items-center space-x-2">
                        <span class=move || {
                            if !is_connected() {
                                "w-2 h-2 rounded-full bg-[#e01d1d]"
                            } else if protection_paused() {
                                "w-2 h-2 rounded-full bg-[#e9d81d]"
                            } else {
                                "w-2 h-2 rounded-full bg-[#1de966]"
                            }
                        }></span>
                        <div class="min-w-0">
                            <div class="text-[11px] font-medium text-white truncate">"Privacy Filter"</div>
                            <div class="text-[9px] text-[#888888]">
                                {move || if !is_connected() { "Offline" } else if protection_paused() { "Paused" } else { "Active" }}
                            </div>
                        </div>
                    </div>

                    // Xray VPN
                    <div class="p-2 rounded bg-[#1b1b1b] border border-[#2c2c2c] flex items-center space-x-2">
                        <span class=move || {
                            if xray_status.get().connected {
                                "w-2 h-2 rounded-full bg-[#1de966] animate-pulse"
                            } else {
                                "w-2 h-2 rounded-full bg-[#888888]"
                            }
                        }></span>
                        <div class="min-w-0">
                            <div class="text-[11px] font-medium text-white truncate">"Xray Gateway"</div>
                            <div class="text-[9px] text-[#888888] truncate">
                                {move || {
                                    let s = xray_status.get();
                                    if s.connected {
                                        s.node_name.unwrap_or_else(|| "Connected".to_string())
                                    } else {
                                        "Ready / Standby".to_string()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // History
                    <div class="p-2 rounded bg-[#1b1b1b] border border-[#2c2c2c] flex items-center space-x-2">
                        <span class=move || {
                            if !is_connected() {
                                "w-2 h-2 rounded-full bg-[#e01d1d]"
                            } else if protection_paused() {
                                "w-2 h-2 rounded-full bg-[#e9d81d]"
                            } else if !history_enabled() {
                                "w-2 h-2 rounded-full bg-[#888888]"
                            } else {
                                "w-2 h-2 rounded-full bg-[#1de966]"
                            }
                        }></span>
                        <div class="min-w-0">
                            <div class="text-[11px] font-medium text-white truncate">"Net History"</div>
                            <div class="text-[9px] text-[#888888]">
                                {move || if !is_connected() { "Offline" } else if protection_paused() { "Paused" } else if history_enabled() { "Logging Active" } else { "Disabled" }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // 3. NETWORK SCOUT (Live Application Accordion List)
            <div class="space-y-2 flex-1 flex flex-col min-h-0">
                <div class="flex items-center justify-between text-[11px] text-[#888888] px-0.5">
                    <span class="font-medium uppercase tracking-wider text-[10px]">"Network Scout"</span>
                    <span class="text-[10px] text-[#ababab]">
                        {move || format!("{} APPS", profiles.get().len())}
                    </span>
                </div>

                // Search & Sort bar
                <div class="flex items-center space-x-1.5">
                    <div class="relative flex-1">
                        <input
                            type="text"
                            placeholder="Search apps..."
                            prop:value=move || scout_search.get()
                            on:input=move |ev| set_scout_search.set(event_target_value(&ev))
                            class="input-pm !py-1 !pl-7 text-xs"
                        />
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                            class="absolute w-3.5 h-3.5 top-2 left-2 text-[#888888]"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </div>

                    <button
                        on:click=move |_| {
                            let next = match sort_by.get().as_str() {
                                "Activity" => "Name",
                                "Name" => "Blocked",
                                _ => "Activity",
                            };
                            set_sort_by.set(next.to_string());
                        }
                        title="Sort Applications"
                        class="btn-pm !py-1 !px-2 text-[10px] flex items-center space-x-1 whitespace-nowrap"
                    >
                        <span>{move || sort_by.get()}</span>
                    </button>
                </div>

                // Applications List
                <div class="flex-1 overflow-y-auto space-y-1 pr-0.5">
                    {
                        let state = state.clone();
                        move || {
                            let query = scout_search.get().to_lowercase();
                            let list = profiles.get();
                            let mut filtered: Vec<_> = list
                                .into_iter()
                                .filter(|p| {
                                    if query.is_empty() {
                                        true
                                    } else {
                                        p.name.to_lowercase().contains(&query)
                                            || p.id.to_lowercase().contains(&query)
                                    }
                                })
                                .collect();

                            match sort_by.get().as_str() {
                                "Name" => filtered.sort_by_key(|a| a.display_name().to_lowercase()),
                                "Blocked" => filtered.sort_by(|a, b| {
                                    let b_bl = b.stats.map(|s| s.blocked_count).unwrap_or(0);
                                    let a_bl = a.stats.map(|s| s.blocked_count).unwrap_or(0);
                                    b_bl.cmp(&a_bl)
                                }),
                                _ => filtered.sort_by(|a, b| {
                                    let b_act = b.stats.map(|s| s.connections_count).unwrap_or(0);
                                    let a_act = a.stats.map(|s| s.connections_count).unwrap_or(0);
                                    b_act.cmp(&a_act)
                                }),
                            }

                            if filtered.is_empty() {
                                return view! {
                                    <div class="p-4 text-center text-xs text-[#888888]">
                                        "No applications matching search."
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
                                    let is_vpn = p.route_via_vpn_with_default(state.route_by_default.get());
                                    let is_blocked = p.default_action() == "block";
                                    let conn_count = p.count_alive_connections();

                                    let s_click = state.clone();
                                    let app_id_click = app_id.clone();
                                    let on_open = move |_| {
                                        s_click.select_app(Some(app_id_click.clone()));
                                        s_click.set_side_dash_open.set(false);
                                    };

                                    view! {
                                        <div
                                            on:click=on_open
                                            class="p-2 rounded bg-[#1b1b1b] hover:bg-[#222222] border border-[#2c2c2c] flex items-center justify-between cursor-pointer transition-colors group"
                                        >
                                            <div class="flex items-center space-x-2.5 min-w-0">
                                                <div class="w-6 h-6 rounded bg-[#2c2c2c] flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0">
                                                    {initials}
                                                </div>
                                                <div class="min-w-0">
                                                    <div class="text-xs font-medium text-white truncate group-hover:text-[#4e97fa] transition-colors">
                                                        {app_name}
                                                    </div>
                                                    <div class="text-[9px] text-[#888888] flex items-center space-x-1.5">
                                                        {if is_blocked {
                                                            view! { <span class="text-[#e01d1d]">"Blocked"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="text-[#1de966]">"Permitted"</span> }.into_any()
                                                        }}
                                                        {if is_vpn {
                                                            view! { <span class="text-[#4e97fa]">"• VPN"</span> }.into_any()
                                                        } else {
                                                            view! { <span/> }.into_any()
                                                        }}
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="flex items-center space-x-1.5 flex-shrink-0">
                                                <span class="px-1.5 py-0.5 rounded bg-[#222222] text-[10px] font-mono text-[#ababab]">
                                                    {conn_count}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }
                </div>
            </div>

            // 4. INTERACTIVE PROMPTS (#2083 Allow All / Block All) & NOTIFICATIONS DRAWER
            <div class="space-y-2 border-t border-[#2c2c2c] pt-2">
                {
                    let s_all_ok = state.clone();
                    let s_all_no = state.clone();
                    let s_prompts = state.clone();
                    let lang = state.lang;
                    move || {
                        let prompts: Vec<_> = notifications.get().into_iter().filter(|n| n.is_prompt).collect();
                        let l = lang.get();
                        if prompts.is_empty() {
                            return view! { <span/> }.into_any();
                        }
                        let s1 = s_all_ok.clone();
                        let s2 = s_all_no.clone();
                        let s_item = s_prompts.clone();
                        view! {
                            <div class="p-2 rounded bg-[#222222] border border-[#e9d31d]/40 space-y-2">
                                <div class="flex items-center justify-between gap-1">
                                    <span class="text-[10px] font-semibold uppercase text-[#e9d31d]">
                                        {format!("Prompts ({})", prompts.len())}
                                    </span>
                                    <div class="flex items-center gap-1">
                                        <button
                                            on:click=move |_| s1.resolve_all_prompts(true)
                                            class="px-1.5 py-0.5 rounded bg-[#143d24] text-[#1de966] text-[10px] font-semibold hover:bg-[#18823d] cursor-pointer"
                                        >
                                            {t(l, "dash.allow_all")}
                                        </button>
                                        <button
                                            on:click=move |_| s2.resolve_all_prompts(false)
                                            class="px-1.5 py-0.5 rounded bg-[#3d1414] text-[#e01d1d] text-[10px] font-semibold hover:bg-[#811818] cursor-pointer"
                                        >
                                            {t(l, "dash.block_all")}
                                        </button>
                                    </div>
                                </div>
                                <div class="space-y-1 max-h-36 overflow-y-auto">
                                    {prompts.into_iter().map(|p| {
                                        let id_ok = p.id.clone();
                                        let id_no = p.id.clone();
                                        let so = s_item.clone();
                                        let sn = s_item.clone();
                                        view! {
                                            <div class="p-1.5 rounded bg-[#1b1b1b] border border-[#2c2c2c] space-y-1">
                                                <div class="text-[11px] font-semibold text-white truncate">{p.title}</div>
                                                <div class="text-[10px] text-[#ababab] truncate">{p.message}</div>
                                                <div class="flex items-center gap-1.5">
                                                    <button
                                                        on:click=move |_| so.resolve_prompt_id(id_ok.clone(), true)
                                                        class="px-2 py-0.5 rounded bg-[#143d24] text-[#1de966] text-[10px] cursor-pointer"
                                                    >
                                                        {t(l, "dash.allow")}
                                                    </button>
                                                    <button
                                                        on:click=move |_| sn.resolve_prompt_id(id_no.clone(), false)
                                                        class="px-2 py-0.5 rounded bg-[#3d1414] text-[#e01d1d] text-[10px] cursor-pointer"
                                                    >
                                                        {t(l, "dash.block")}
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }

                <div class="flex items-center justify-between text-[11px] text-[#888888] px-0.5">
                    <span class="font-medium uppercase tracking-wider text-[10px]">
                        {let lang = state.lang; move || t(lang.get(), "dash.alerts")}
                    </span>
                    <button
                        on:click={
                            let s_clear = state.clone();
                            move |_| s_clear.clear_notifications()
                        }
                        class="text-[10px] text-[#888888] hover:text-white transition-colors"
                    >
                        {let lang = state.lang; move || t(lang.get(), "dash.clear")}
                    </button>
                </div>

                <div class="space-y-1.5 max-h-36 overflow-y-auto">
                    {move || {
                        let items: Vec<_> = notifications.get().into_iter().filter(|n| !n.is_prompt).collect();
                        let l = state.lang.get();
                        if items.is_empty() {
                            return view! {
                                <div class="text-[11px] text-[#888888] p-2 text-center">
                                    {t(l, "dash.no_alerts")}
                                </div>
                            }.into_any();
                        }

                        items.into_iter().map(|n| {
                            let badge_color = match n.level {
                                "warning" => "text-[#e9d31d] bg-[#e9d31d]/10 border-[#e9d31d]/20",
                                "error" => "text-[#e01d1d] bg-[#e01d1d]/10 border-[#e01d1d]/20",
                                _ => "text-[#4e97fa] bg-[#4e97fa]/10 border-[#4e97fa]/20",
                            };

                            view! {
                                <div class="p-2 rounded bg-[#1b1b1b] border border-[#2c2c2c] space-y-1">
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center space-x-1.5 truncate">
                                            <span class={format!("px-1.5 py-0.2 rounded text-[8px] font-mono border uppercase tracking-wider {}", badge_color)}>
                                                {n.level}
                                            </span>
                                            <span class="font-semibold text-white truncate text-xs">{n.title}</span>
                                        </div>
                                        <span class="text-[9px] text-[#888888] whitespace-nowrap ml-2">{n.time}</span>
                                    </div>
                                    <p class="text-[11px] text-[#ababab] leading-tight">{n.message}</p>
                                </div>
                            }
                        }).collect_view().into_any()
                    }}
                </div>
            </div>
        </aside>
    }
}
