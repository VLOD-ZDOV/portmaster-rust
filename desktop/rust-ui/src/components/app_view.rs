//! Detailed Application View matching Portmaster 1:1.
//!
//! Tabs: Rules & Routing, App-Specific Settings, Live Connections, Details.

use leptos::prelude::*;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Rules,
    Settings,
    Connections,
    Details,
}

#[component]
pub fn AppDetailView(app_id: String) -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let (active_tab, set_active_tab) = signal(AppTab::Rules);

    // Endpoint rules list local state
    let (new_rule, set_new_rule) = signal(String::new());

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
        <div class="space-y-6">
            // Back button and Breadcrumb
            <div class="flex items-center space-x-2 text-xs text-slate-400">
                <button
                    on:click=on_back
                    class="hover:text-white flex items-center space-x-1 font-medium transition-colors"
                >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
                    </svg>
                    <span>"Applications"</span>
                </button>
                <span>"/"</span>
                <span class="text-white font-semibold">
                    {move || profile.get().map(|p| p.display_name().to_string()).unwrap_or_else(|| app_id.clone())}
                </span>
            </div>

            {move || {
                let Some(p) = profile.get() else {
                    return view! {
                        <div class="p-12 text-center rounded-2xl bg-slate-900 border border-slate-800 text-slate-400">
                            "Application profile not found."
                        </div>
                    }.into_any();
                };

                let app_id = p.id.clone();
                let is_vpn = p.route_via_vpn();
                let action = p.default_action().to_string();
                let display_name = p.display_name().to_string();
                let path = p.presentation_path.clone();
                let monogram = display_name.chars().next().unwrap_or('A').to_uppercase().to_string();

                let state_toggle = state.clone();
                let id_for_vpn = app_id.clone();

                let state_permit = state.clone();
                let id_for_permit = app_id.clone();

                let state_block = state.clone();
                let id_for_block = app_id.clone();

                let state_ask = state.clone();
                let id_for_ask = app_id.clone();

                view! {
                    <div class="space-y-6">
                        // Application Header & Stats
                        <div class="p-6 rounded-2xl bg-slate-900/80 border border-slate-800 shadow-xl space-y-6">
                            <div class="flex items-center justify-between flex-wrap gap-4">
                                // App identity
                                <div class="flex items-center space-x-4">
                                    <div class=if is_vpn {
                                        "w-14 h-14 rounded-2xl bg-cyan-500/20 text-cyan-400 border border-cyan-500/40 flex items-center justify-center font-bold text-xl shadow-lg shadow-cyan-500/10"
                                    } else {
                                        "w-14 h-14 rounded-2xl bg-slate-800 text-slate-200 border border-slate-700 flex items-center justify-center font-bold text-xl"
                                    }>
                                        {monogram}
                                    </div>
                                    <div>
                                        <div class="flex items-center space-x-2">
                                            <h2 class="text-xl font-bold text-white tracking-tight">{display_name}</h2>
                                            {if is_vpn {
                                                view! {
                                                    <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
                                                        "VPN Tunnel Active"
                                                    </span>
                                                }.into_any()
                                            } else {
                                                view! { <span/> }.into_any()
                                            }}
                                        </div>
                                        <p class="text-xs font-mono text-slate-400 mt-1">{path.clone()}</p>
                                    </div>
                                </div>

                                // Action Buttons (Permit / Ask / Block)
                                <div class="flex items-center space-x-3">
                                    <div class="flex items-center rounded-xl bg-slate-950 p-1 border border-slate-800 text-xs font-medium">
                                        <button
                                            on:click=move |_| state_permit.set_app_action(id_for_permit.clone(), "permit".to_string())
                                            class=if action == "permit" {
                                                "px-3 py-1.5 rounded-lg bg-emerald-500/20 text-emerald-300 font-semibold border border-emerald-500/30"
                                            } else {
                                                "px-3 py-1.5 text-slate-400 hover:text-slate-200 transition-colors"
                                            }
                                        >
                                            "Permit"
                                        </button>
                                        <button
                                            on:click=move |_| state_ask.set_app_action(id_for_ask.clone(), "ask".to_string())
                                            class=if action == "ask" {
                                                "px-3 py-1.5 rounded-lg bg-amber-500/20 text-amber-300 font-semibold border border-amber-500/30"
                                            } else {
                                                "px-3 py-1.5 text-slate-400 hover:text-slate-200 transition-colors"
                                            }
                                        >
                                            "Ask"
                                        </button>
                                        <button
                                            on:click=move |_| state_block.set_app_action(id_for_block.clone(), "block".to_string())
                                            class=if action == "block" {
                                                "px-3 py-1.5 rounded-lg bg-rose-500/20 text-rose-300 font-semibold border border-rose-500/30"
                                            } else {
                                                "px-3 py-1.5 text-slate-400 hover:text-slate-200 transition-colors"
                                            }
                                        >
                                            "Block"
                                        </button>
                                    </div>

                                    // VPN Toggle
                                    <div class="flex items-center space-x-2 pl-2 border-l border-slate-800">
                                        <span class="text-xs text-slate-400">"VPN"</span>
                                        <button
                                            on:click=move |_| state_toggle.toggle_app_vpn(id_for_vpn.clone())
                                            role="switch"
                                            aria-checked=if is_vpn { "true" } else { "false" }
                                            class=if is_vpn {
                                                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-cyan-500 transition-colors duration-200 shadow-md shadow-cyan-500/20"
                                            } else {
                                                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-slate-800 transition-colors duration-200"
                                            }
                                        >
                                            <span
                                                class=if is_vpn {
                                                    "translate-x-5 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200"
                                                } else {
                                                    "translate-x-0 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-slate-400 shadow ring-0 transition duration-200"
                                                }
                                            />
                                        </button>
                                    </div>
                                </div>
                            </div>

                            // Stats Grid
                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-4 border-t border-slate-800/80 text-center">
                                <div class="p-3 rounded-xl bg-slate-950/60 border border-slate-800/80">
                                    <div class="text-base font-bold text-white font-mono">"48"</div>
                                    <div class="text-[11px] text-slate-400">"Total Connections"</div>
                                </div>
                                <div class="p-3 rounded-xl bg-slate-950/60 border border-slate-800/80">
                                    <div class="text-base font-bold text-emerald-400 font-mono">"8.3%"</div>
                                    <div class="text-[11px] text-slate-400">"Blocked Rate"</div>
                                </div>
                                <div class="p-3 rounded-xl bg-slate-950/60 border border-slate-800/80">
                                    <div class="text-base font-bold text-blue-400 font-mono">"12.4 MB"</div>
                                    <div class="text-[11px] text-slate-400">"Received"</div>
                                </div>
                                <div class="p-3 rounded-xl bg-slate-950/60 border border-slate-800/80">
                                    <div class="text-base font-bold text-indigo-400 font-mono">"2.1 MB"</div>
                                    <div class="text-[11px] text-slate-400">"Sent"</div>
                                </div>
                            </div>
                        </div>

                        // Tabs Navigation (Rules & Routing, Settings, Connections, Details)
                        <div class="flex items-center space-x-2 border-b border-slate-800 pb-2 text-xs font-semibold">
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Rules)
                                class=move || {
                                    if active_tab.get() == AppTab::Rules {
                                        "px-4 py-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30"
                                    } else {
                                        "px-4 py-2 rounded-xl text-slate-400 hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Rules & Routing"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Settings)
                                class=move || {
                                    if active_tab.get() == AppTab::Settings {
                                        "px-4 py-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30"
                                    } else {
                                        "px-4 py-2 rounded-xl text-slate-400 hover:text-white transition-colors"
                                    }
                                }
                            >
                                "App Settings"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Connections)
                                class=move || {
                                    if active_tab.get() == AppTab::Connections {
                                        "px-4 py-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30"
                                    } else {
                                        "px-4 py-2 rounded-xl text-slate-400 hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Live Connections"
                            </button>
                            <button
                                on:click=move |_| set_active_tab.set(AppTab::Details)
                                class=move || {
                                    if active_tab.get() == AppTab::Details {
                                        "px-4 py-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30"
                                    } else {
                                        "px-4 py-2 rounded-xl text-slate-400 hover:text-white transition-colors"
                                    }
                                }
                            >
                                "Details"
                            </button>
                        </div>

                        // TAB CONTENT
                        <div>
                            {move || {
                                match active_tab.get() {
                                    AppTab::Rules => view! {
                                        <div class="space-y-4">
                                            <div class="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
                                                <div>
                                                    <h3 class="text-sm font-bold text-white">"Endpoint Rules for this Application"</h3>
                                                    <p class="text-xs text-slate-400 mt-1">
                                                        "Define granular firewall rules for specific domains and IP addresses contacted by this application."
                                                    </p>
                                                </div>

                                                // Sample App rules
                                                <div class="space-y-2">
                                                    <div class="flex items-center justify-between p-2.5 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono">
                                                        <div class="flex items-center space-x-2">
                                                            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
                                                            <span class="text-slate-200">"api.telegram.org"</span>
                                                        </div>
                                                        <span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-400 font-bold">
                                                            "PERMIT"
                                                        </span>
                                                    </div>
                                                    <div class="flex items-center justify-between p-2.5 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono">
                                                        <div class="flex items-center space-x-2">
                                                            <span class="w-2 h-2 rounded-full bg-rose-400"></span>
                                                            <span class="text-slate-200">"crashlytics.com"</span>
                                                        </div>
                                                        <span class="px-2 py-0.5 rounded text-[10px] bg-rose-500/10 text-rose-400 font-bold">
                                                            "BLOCK"
                                                        </span>
                                                    </div>
                                                </div>

                                                // Add rule
                                                <div class="flex items-center space-x-2 pt-2 border-t border-slate-800">
                                                    <input
                                                        type="text"
                                                        placeholder="Add rule: domain.com or IP [permit/block]..."
                                                        prop:value=move || new_rule.get()
                                                        on:input=move |ev| set_new_rule.set(event_target_value(&ev))
                                                        class="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-cyan-500"
                                                    />
                                                    <button class="px-4 py-2 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-semibold text-xs transition-colors">
                                                        "Add Rule"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any(),

                                    AppTab::Settings => view! {
                                        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 text-xs space-y-4">
                                            <h3 class="text-sm font-bold text-white">"Application Configuration Overrides"</h3>
                                            <p class="text-slate-400 leading-relaxed">
                                                "By default, this application inherits all configuration from Global Settings. Any setting modified here acts as an exception."
                                            </p>
                                            <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-between">
                                                <div>
                                                    <div class="font-semibold text-white">"Inherit Global Firewall Defaults"</div>
                                                    <div class="text-slate-400 text-[11px]">"Use global rules, filter lists, and DNS resolvers"</div>
                                                </div>
                                                <button class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 font-medium">
                                                    "Reset All to Global"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any(),

                                    AppTab::Connections => view! {
                                        <div class="rounded-2xl bg-slate-900/60 border border-slate-800 overflow-hidden text-xs">
                                            <table class="w-full text-left">
                                                <thead class="bg-slate-950/80 text-[10px] uppercase font-bold text-slate-400 border-b border-slate-800">
                                                    <tr>
                                                        <th class="p-3">"Verdict"</th>
                                                        <th class="p-3">"Destination"</th>
                                                        <th class="p-3">"Port"</th>
                                                        <th class="p-3">"Protocol"</th>
                                                        <th class="p-3">"Country"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-slate-800 font-mono text-[11px]">
                                                    <tr class="hover:bg-slate-800/40 transition-colors">
                                                        <td class="p-3"><span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-400 font-bold">"PERMIT"</span></td>
                                                        <td class="p-3 text-white">"149.154.167.50"</td>
                                                        <td class="p-3 text-slate-400">"443"</td>
                                                        <td class="p-3 text-slate-400">"TCP / TLS"</td>
                                                        <td class="p-3 text-cyan-400 font-bold">"NL"</td>
                                                    </tr>
                                                    <tr class="hover:bg-slate-800/40 transition-colors">
                                                        <td class="p-3"><span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-400 font-bold">"PERMIT"</span></td>
                                                        <td class="p-3 text-white">"91.108.56.172"</td>
                                                        <td class="p-3 text-slate-400">"443"</td>
                                                        <td class="p-3 text-slate-400">"TCP / TLS"</td>
                                                        <td class="p-3 text-cyan-400 font-bold">"DE"</td>
                                                    </tr>
                                                    <tr class="hover:bg-slate-800/40 transition-colors">
                                                        <td class="p-3"><span class="px-2 py-0.5 rounded text-[10px] bg-rose-500/10 text-rose-400 font-bold">"BLOCK"</span></td>
                                                        <td class="p-3 text-rose-300">"app-measurement.com"</td>
                                                        <td class="p-3 text-slate-400">"443"</td>
                                                        <td class="p-3 text-slate-400">"TCP / HTTPS"</td>
                                                        <td class="p-3 text-slate-400">"US"</td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any(),

                                    AppTab::Details => view! {
                                        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 text-xs space-y-4">
                                            <h3 class="text-sm font-bold text-white">"Profile Metadata"</h3>
                                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                                <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-1">
                                                    <span class="text-slate-400 text-[10px] uppercase font-semibold">"Profile ID"</span>
                                                    <div class="font-mono text-white">{app_id.clone()}</div>
                                                </div>
                                                <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-1">
                                                    <span class="text-slate-400 text-[10px] uppercase font-semibold">"Source"</span>
                                                    <div class="font-mono text-white">"local"</div>
                                                </div>
                                                <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-1 sm:col-span-2">
                                                    <span class="text-slate-400 text-[10px] uppercase font-semibold">"Presentation Path"</span>
                                                    <div class="font-mono text-white break-all">{path.clone()}</div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any(),
                                }
                            }}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
