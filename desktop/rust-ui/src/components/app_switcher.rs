//! Application switcher & VPN split-tunneling router component.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn AppSwitcher() -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let filter_query = state.filter_query;
    let set_filter_query = state.set_filter_query;

    // Filter tab: "all", "vpn", "direct"
    let (active_tab, set_active_tab) = signal("all".to_string());

    view! {
        <section class="bg-slate-900/60 border border-slate-800 rounded-2xl p-6 shadow-xl backdrop-blur-sm transition-all hover:border-slate-700/80 mt-6">
            // Header with search and counts
            <div class="flex items-center justify-between pb-5 border-b border-slate-800/80 mb-5 flex-wrap gap-4">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400">
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                        </svg>
                    </div>
                    <div>
                        <div class="flex items-center space-x-3">
                            <h2 class="text-lg font-semibold text-white tracking-tight">"App Routing & Protection"</h2>
                            <span class="px-2 py-0.5 text-xs font-semibold rounded-full bg-slate-800 text-slate-300 border border-slate-700">
                                {move || format!("{} Apps", profiles.get().len())}
                            </span>
                        </div>
                        <p class="text-xs text-slate-400">"Control per-app VPN routing and firewall behavior"</p>
                    </div>
                </div>

                // Search & Filter controls
                <div class="flex items-center space-x-3 flex-wrap gap-2">
                    // Search box
                    <div class="relative">
                        <input
                            type="text"
                            placeholder="Filter apps..."
                            prop:value=move || filter_query.get()
                            on:input=move |ev| {
                                set_filter_query.set(event_target_value(&ev));
                            }
                            class="w-48 sm:w-64 pl-9 pr-3 py-1.5 rounded-xl bg-slate-950/80 border border-slate-700/80 text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500 transition-all"
                        />
                        <svg class="w-4 h-4 text-slate-500 absolute left-3 top-2.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                    </div>

                    // Tabs: All / VPN / Direct
                    <div class="flex items-center rounded-xl bg-slate-950 p-1 border border-slate-800 text-xs font-medium">
                        <button
                            on:click=move |_| set_active_tab.set("all".to_string())
                            class=move || {
                                if active_tab.get() == "all" {
                                    "px-3 py-1 rounded-lg bg-slate-800 text-white shadow-sm"
                                } else {
                                    "px-3 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "All"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("vpn".to_string())
                            class=move || {
                                if active_tab.get() == "vpn" {
                                    "px-3 py-1 rounded-lg bg-cyan-600/30 text-cyan-300 shadow-sm border border-cyan-500/30"
                                } else {
                                    "px-3 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "VPN Routed"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("direct".to_string())
                            class=move || {
                                if active_tab.get() == "direct" {
                                    "px-3 py-1 rounded-lg bg-slate-800 text-white shadow-sm"
                                } else {
                                    "px-3 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "Direct"
                        </button>
                    </div>
                </div>
            </div>

            // App list
            <div class="space-y-3">
                {move || {
                    let query = filter_query.get().to_lowercase();
                    let tab = active_tab.get();
                    let list = profiles.get();

                    let filtered: Vec<_> = list.into_iter().filter(|p| {
                        // Text match
                        let name_match = p.name.to_lowercase().contains(&query)
                            || p.presentation_path.to_lowercase().contains(&query)
                            || p.id.to_lowercase().contains(&query);

                        if !name_match {
                            return false;
                        }

                        // Tab filter
                        match tab.as_str() {
                            "vpn" => p.route_via_vpn(),
                            "direct" => !p.route_via_vpn(),
                            _ => true,
                        }
                    }).collect();

                    if filtered.is_empty() {
                        view! {
                            <div class="p-8 text-center rounded-xl bg-slate-950/40 border border-slate-800/60 text-slate-500 text-sm">
                                "No applications found matching the filter."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="grid grid-cols-1 gap-3">
                                {filtered.into_iter().map(|p| {
                                    let app_id = p.id.clone();
                                    let is_vpn = p.route_via_vpn();
                                    let display_name = p.display_name().to_string();
                                    let path = p.presentation_path.clone();
                                    let action = p.default_action().to_string();

                                    let state_toggle = state.clone();
                                    let id_for_vpn = app_id.clone();

                                    let state_permit = state.clone();
                                    let id_for_permit = app_id.clone();

                                    let state_block = state.clone();
                                    let id_for_block = app_id.clone();

                                    let monogram = display_name.chars().next().unwrap_or('A').to_uppercase().to_string();

                                    view! {
                                        <div class="bg-slate-950/60 rounded-xl p-4 border border-slate-800/80 flex items-center justify-between flex-wrap gap-4 transition-all hover:border-slate-700">
                                            // Left: Monogram + Name + Path
                                            <div class="flex items-center space-x-3 min-w-[200px]">
                                                <div class=if is_vpn {
                                                    "w-10 h-10 rounded-xl bg-cyan-500/20 text-cyan-400 border border-cyan-500/40 flex items-center justify-center font-bold text-sm shadow-sm"
                                                } else {
                                                    "w-10 h-10 rounded-xl bg-slate-800 text-slate-300 border border-slate-700 flex items-center justify-center font-bold text-sm"
                                                }>
                                                    {monogram}
                                                </div>
                                                <div>
                                                    <div class="flex items-center space-x-2">
                                                        <h4 class="text-sm font-semibold text-white tracking-tight">{display_name}</h4>
                                                        {if is_vpn {
                                                            view! {
                                                                <span class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                                                                    "VPN"
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span/> }.into_any()
                                                        }}
                                                    </div>
                                                    <p class="text-xs text-slate-500 font-mono truncate max-w-xs">{path}</p>
                                                </div>
                                            </div>

                                            // Middle: Firewall Action selector (Permit / Ask / Block)
                                            <div class="flex items-center rounded-lg bg-slate-900 p-0.5 border border-slate-800 text-xs">
                                                <button
                                                    on:click=move |_| state_permit.set_app_action(id_for_permit.clone(), "permit".to_string())
                                                    class=if action == "permit" {
                                                        "px-2.5 py-1 rounded-md bg-emerald-500/20 text-emerald-300 font-semibold border border-emerald-500/30"
                                                    } else {
                                                        "px-2.5 py-1 text-slate-400 hover:text-slate-200 transition-colors"
                                                    }
                                                >
                                                    "Permit"
                                                </button>
                                                <button
                                                    on:click=move |_| state_block.set_app_action(id_for_block.clone(), "block".to_string())
                                                    class=if action == "block" {
                                                        "px-2.5 py-1 rounded-md bg-rose-500/20 text-rose-300 font-semibold border border-rose-500/30"
                                                    } else {
                                                        "px-2.5 py-1 text-slate-400 hover:text-slate-200 transition-colors"
                                                    }
                                                >
                                                    "Block"
                                                </button>
                                            </div>

                                            // Right: Route via VPN interactive Switch
                                            <div class="flex items-center space-x-3">
                                                <span class="text-xs text-slate-400 font-medium">
                                                    {if is_vpn { "Tunnel via VPN" } else { "Direct (Bypass)" }}
                                                </span>
                                                <button
                                                    on:click=move |_| state_toggle.toggle_app_vpn(id_for_vpn.clone())
                                                    role="switch"
                                                    aria-checked=if is_vpn { "true" } else { "false" }
                                                    class=if is_vpn {
                                                        "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-cyan-500 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-cyan-500/50 shadow-md shadow-cyan-500/20"
                                                    } else {
                                                        "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-slate-800 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-slate-700"
                                                    }
                                                >
                                                    <span
                                                        class=if is_vpn {
                                                            "translate-x-5 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out"
                                                        } else {
                                                            "translate-x-0 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-slate-400 shadow-lg ring-0 transition duration-200 ease-in-out"
                                                        }
                                                    />
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}
