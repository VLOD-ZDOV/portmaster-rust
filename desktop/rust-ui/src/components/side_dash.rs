//! Slide-out SideDash drawer with security status, feature scout, and network scout.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn SideDash() -> impl IntoView {
    let state = expect_context::<AppState>();
    let is_open = state.is_side_dash_open;
    let security_level = state.security_level;
    let set_security_level = state.set_security_level;
    let notifications = state.notifications;
    let xray_status = state.xray_status;

    let state_close = state.clone();
    let on_close = move |_| {
        state_close.set_side_dash_open.set(false);
    };

    view! {
        <aside class=move || {
            if is_open.get() {
                "fixed top-0 left-16 bottom-0 w-80 bg-slate-900/95 backdrop-blur-xl border-r border-slate-800 z-30 flex flex-col shadow-2xl transition-transform duration-300 ease-in-out translate-x-0"
            } else {
                "fixed top-0 left-16 bottom-0 w-80 bg-slate-900/95 backdrop-blur-xl border-r border-slate-800 z-30 flex flex-col shadow-2xl transition-transform duration-300 ease-in-out -translate-x-full pointer-events-none"
            }
        }>
            // Header
            <div class="p-4 border-b border-slate-800 flex items-center justify-between">
                <div class="flex items-center space-x-2">
                    <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
                    <h2 class="text-sm font-bold text-white tracking-tight">"Portmaster Dashboard"</h2>
                </div>
                <button
                    on:click=on_close
                    title="Close Drawer"
                    class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            // Scrollable Content
            <div class="flex-1 overflow-y-auto p-4 space-y-5 text-xs">
                // 1. Security Lock Status
                <div class="p-3.5 rounded-xl bg-slate-950/70 border border-slate-800/80">
                    <div class="flex items-center space-x-3 mb-2.5">
                        <div class="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400">
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                            </svg>
                        </div>
                        <div>
                            <span class="text-[10px] uppercase font-semibold text-slate-400">"Security Policy"</span>
                            <div class="font-bold text-white">{move || security_level.get()}</div>
                        </div>
                    </div>

                    // Quick mode buttons
                    <div class="grid grid-cols-3 gap-1 pt-2 border-t border-slate-800/60 text-[10px] font-medium text-center">
                        <button
                            on:click=move |_| set_security_level.set("Standard (Balanced)".to_string())
                            class="py-1 px-1.5 rounded bg-slate-800/80 hover:bg-slate-800 text-slate-300 transition-colors"
                        >
                            "Standard"
                        </button>
                        <button
                            on:click=move |_| set_security_level.set("Strict (High Privacy)".to_string())
                            class="py-1 px-1.5 rounded bg-slate-800/80 hover:bg-slate-800 text-slate-300 transition-colors"
                        >
                            "Strict"
                        </button>
                        <button
                            on:click=move |_| set_security_level.set("Block All".to_string())
                            class="py-1 px-1.5 rounded bg-slate-800/80 hover:bg-slate-800 text-rose-300 transition-colors"
                        >
                            "Lockdown"
                        </button>
                    </div>
                </div>

                // 2. Feature Scout / System Health
                <div class="p-3.5 rounded-xl bg-slate-950/70 border border-slate-800/80 space-y-2">
                    <span class="text-[10px] uppercase font-semibold text-slate-400 block mb-1">"Active Subsystems"</span>

                    <div class="flex items-center justify-between py-1 border-b border-slate-800/50">
                        <span class="text-slate-300">"Core Engine"</span>
                        <span class="text-emerald-400 font-medium flex items-center">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 mr-1.5"></span>
                            "Online"
                        </span>
                    </div>

                    <div class="flex items-center justify-between py-1 border-b border-slate-800/50">
                        <span class="text-slate-300">"Packet Interceptor"</span>
                        <span class="text-emerald-400 font-medium flex items-center">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 mr-1.5"></span>
                            "Filtering"
                        </span>
                    </div>

                    <div class="flex items-center justify-between py-1 border-b border-slate-800/50">
                        <span class="text-slate-300">"Secure DNS Resolver"</span>
                        <span class="text-emerald-400 font-medium flex items-center">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 mr-1.5"></span>
                            "DoT Active"
                        </span>
                    </div>

                    <div class="flex items-center justify-between py-1">
                        <span class="text-slate-300">"VPN Gateway (Xray)"</span>
                        {move || {
                            let s = xray_status.get();
                            if s.connected {
                                view! {
                                    <span class="text-cyan-400 font-medium flex items-center">
                                        <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 mr-1.5"></span>
                                        "Tunnel Active"
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="text-slate-500 font-medium">"Disconnected"</span>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // 3. Network Scout Mini Graphs
                <div class="p-3.5 rounded-xl bg-slate-950/70 border border-slate-800/80">
                    <span class="text-[10px] uppercase font-semibold text-slate-400 block mb-2">"Network Scout"</span>
                    <div class="grid grid-cols-2 gap-2 text-center mb-3">
                        <div class="p-2 rounded-lg bg-slate-900 border border-slate-800">
                            <div class="text-base font-bold text-white font-mono">"182"</div>
                            <div class="text-[10px] text-slate-400">"Total Conns"</div>
                        </div>
                        <div class="p-2 rounded-lg bg-slate-900 border border-slate-800">
                            <div class="text-base font-bold text-rose-400 font-mono">"14.8%"</div>
                            <div class="text-[10px] text-slate-400">"Blocked"</div>
                        </div>
                    </div>

                    // Visual bar
                    <div class="space-y-1">
                        <div class="flex justify-between text-[10px] text-slate-400">
                            <span>"Permitted vs Blocked"</span>
                            <span class="text-emerald-400 font-mono">"155 / 27"</span>
                        </div>
                        <div class="w-full h-2 rounded-full bg-slate-800 overflow-hidden flex">
                            <div class="h-full bg-emerald-500" style="width: 85%"></div>
                            <div class="h-full bg-rose-500" style="width: 15%"></div>
                        </div>
                    </div>
                </div>

                // 4. Live Notifications
                <div class="p-3.5 rounded-xl bg-slate-950/70 border border-slate-800/80 space-y-2">
                    <div class="flex items-center justify-between">
                        <span class="text-[10px] uppercase font-semibold text-slate-400">"Security Events"</span>
                        <span class="text-[10px] text-cyan-400 font-medium">"Live"</span>
                    </div>

                    <div class="space-y-2 pt-1">
                        {move || {
                            let items = notifications.get();
                            items.into_iter().map(|n| {
                                let badge_color = match n.level {
                                    "warning" => "text-amber-400 bg-amber-400/10 border-amber-400/20",
                                    "error" => "text-rose-400 bg-rose-400/10 border-rose-400/20",
                                    _ => "text-cyan-400 bg-cyan-400/10 border-cyan-400/20",
                                };

                                view! {
                                    <div class="p-2 rounded-lg bg-slate-900 border border-slate-800/80 space-y-1">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center space-x-1.5 truncate">
                                                <span class={format!("px-1.5 py-0.2 rounded text-[8px] font-mono border uppercase tracking-wider {}", badge_color)}>{n.level}</span>
                                                <span class="font-semibold text-white truncate text-xs">{n.title}</span>
                                            </div>
                                            <span class="text-[9px] text-slate-500 whitespace-nowrap ml-2">{n.time}</span>
                                        </div>
                                        <p class="text-[11px] text-slate-400 leading-tight">{n.message}</p>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
            </div>
        </aside>
    }
}
