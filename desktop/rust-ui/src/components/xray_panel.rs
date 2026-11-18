//! VPN (Xray) control panel component.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn XrayPanel() -> impl IntoView {
    let state = expect_context::<AppState>();
    let xray_status = state.xray_status;
    let xray_nodes = state.xray_nodes;
    let is_busy = state.is_busy;

    let on_toggle_vpn = {
        let state = state.clone();
        move |_| {
            state.toggle_vpn();
        }
    };

    view! {
        <section class="bg-slate-900/60 border border-slate-800 rounded-2xl p-6 shadow-xl backdrop-blur-sm transition-all hover:border-slate-700/80">
            // Section Header
            <div class="flex items-center justify-between pb-5 border-b border-slate-800/80 mb-5 flex-wrap gap-4">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                        </svg>
                    </div>
                    <div>
                        <h2 class="text-lg font-semibold text-white tracking-tight">"VPN Gateway (Xray)"</h2>
                        <p class="text-xs text-slate-400">"Encrypted split-tunneling & proxy routing"</p>
                    </div>
                </div>

                // Connect / Disconnect button
                <div class="flex items-center space-x-3">
                    <button
                        on:click=on_toggle_vpn
                        disabled=move || is_busy.get()
                        class=move || {
                            let s = xray_status.get();
                            if is_busy.get() {
                                "px-5 py-2.5 rounded-xl font-medium text-sm transition-all shadow-md bg-slate-800 text-slate-400 cursor-not-allowed flex items-center space-x-2"
                            } else if s.connected {
                                "px-5 py-2.5 rounded-xl font-medium text-sm transition-all shadow-md bg-rose-500 hover:bg-rose-600 text-white shadow-rose-500/20 active:scale-95 flex items-center space-x-2"
                            } else {
                                "px-5 py-2.5 rounded-xl font-medium text-sm transition-all shadow-md bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-400 hover:to-teal-500 text-white shadow-emerald-500/20 active:scale-95 flex items-center space-x-2"
                            }
                        }
                    >
                        {move || {
                            let s = xray_status.get();
                            if is_busy.get() {
                                view! {
                                    <svg class="animate-spin w-4 h-4 text-slate-400" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    <span>"Working..."</span>
                                }.into_any()
                            } else if s.connected {
                                view! {
                                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                    <span>"Disconnect"</span>
                                }.into_any()
                            } else {
                                view! {
                                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                    </svg>
                                    <span>"Connect VPN"</span>
                                }.into_any()
                            }
                        }}
                    </button>
                </div>
            </div>

            // Main status card
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                // Card 1: Active node info
                <div class="md:col-span-2 bg-slate-950/50 rounded-xl p-4 border border-slate-800/80 flex flex-col justify-between">
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <span class="text-xs uppercase tracking-wider text-slate-400 font-semibold">"Current Endpoint"</span>
                            {move || {
                                let s = xray_status.get();
                                if s.connected {
                                    view! {
                                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 mr-1.5 animate-pulse"></span>
                                            "Connected"
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-slate-800 text-slate-400 border border-slate-700">
                                            "Disconnected"
                                        </span>
                                    }.into_any()
                                }
                            }}
                        </div>

                        {move || {
                            let s = xray_status.get();
                            let node_name = s.node_name.unwrap_or_else(|| "No node selected".to_string());
                            let country = s.country.unwrap_or_else(|| "??".to_string());
                            let exit_ip = s.exit_ip.unwrap_or_else(|| "Protected (Direct)".to_string());

                            view! {
                                <div class="flex items-center space-x-3 my-2">
                                    <div class="w-10 h-10 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center font-bold text-sm text-cyan-400 tracking-wider">
                                        {country}
                                    </div>
                                    <div>
                                        <h3 class="text-base font-semibold text-white">{node_name}</h3>
                                        <p class="text-xs text-slate-400 flex items-center space-x-2">
                                            <span>"Exit IP: "</span>
                                            <span class="font-mono text-cyan-300 font-medium">{exit_ip}</span>
                                        </p>
                                    </div>
                                </div>
                            }
                        }}
                    </div>

                    // Quick Node Selector pills
                    <div class="pt-3 border-t border-slate-800/60 mt-3">
                        <span class="text-xs text-slate-400 block mb-2 font-medium">"Switch Location:"</span>
                        <div class="flex items-center space-x-2 overflow-x-auto pb-1 scrollbar-none">
                            {move || {
                                let current_id = xray_status.get().node_id;
                                let nodes = xray_nodes.get();
                                nodes.into_iter().map(|node| {
                                    let node_id = node.id.clone();
                                    let is_active = current_id.as_ref() == Some(&node_id);
                                    let state = state.clone();
                                    let id_for_click = node_id.clone();

                                    view! {
                                        <button
                                            on:click=move |_| state.select_node(id_for_click.clone())
                                            class=if is_active {
                                                "px-3 py-1 rounded-lg text-xs font-semibold bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 flex items-center space-x-1.5 whitespace-nowrap shadow-sm"
                                            } else {
                                                "px-3 py-1 rounded-lg text-xs font-medium bg-slate-800/80 hover:bg-slate-800 text-slate-300 border border-slate-700/60 flex items-center space-x-1.5 whitespace-nowrap transition-colors"
                                            }
                                        >
                                            <span class="font-bold text-xs">{node.country_code}</span>
                                            <span>{node.name}</span>
                                            {if let Some(ms) = node.ping_ms {
                                                view! {
                                                    <span class="text-[10px] opacity-70 font-mono">{format!("{}ms", ms)}</span>
                                                }.into_any()
                                            } else {
                                                view! { <span/> }.into_any()
                                            }}
                                        </button>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </div>

                // Card 2: Live Traffic Counters
                <div class="bg-slate-950/50 rounded-xl p-4 border border-slate-800/80 flex flex-col justify-between">
                    <span class="text-xs uppercase tracking-wider text-slate-400 font-semibold mb-2 block">"Real-time Traffic"</span>

                    <div class="space-y-3 my-auto">
                        // Uplink
                        <div class="flex items-center justify-between p-2.5 rounded-lg bg-slate-900/80 border border-slate-800">
                            <div class="flex items-center space-x-2">
                                <div class="w-6 h-6 rounded-md bg-blue-500/10 text-blue-400 flex items-center justify-center">
                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 10l7-7m0 0l7 7m-7-7v18" />
                                    </svg>
                                </div>
                                <span class="text-xs text-slate-300 font-medium">"Uplink (Sent)"</span>
                            </div>
                            <span class="text-sm font-bold font-mono text-blue-400">
                                {move || {
                                    xray_status.get().traffic
                                        .map(|t| t.formatted_uplink())
                                        .unwrap_or_else(|| "0 B".to_string())
                                }}
                            </span>
                        </div>

                        // Downlink
                        <div class="flex items-center justify-between p-2.5 rounded-lg bg-slate-900/80 border border-slate-800">
                            <div class="flex items-center space-x-2">
                                <div class="w-6 h-6 rounded-md bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                                    </svg>
                                </div>
                                <span class="text-xs text-slate-300 font-medium">"Downlink (Received)"</span>
                            </div>
                            <span class="text-sm font-bold font-mono text-emerald-400">
                                {move || {
                                    xray_status.get().traffic
                                        .map(|t| t.formatted_downlink())
                                        .unwrap_or_else(|| "0 B".to_string())
                                }}
                            </span>
                        </div>
                    </div>

                    <div class="text-[11px] text-slate-500 pt-2 border-t border-slate-800/60 flex items-center justify-between">
                        <span>"Protocol: VLESS / XTLS"</span>
                        <span class="text-emerald-500/80 font-medium">"Encrypted"</span>
                    </div>
                </div>
            </div>
        </section>
    }
}
