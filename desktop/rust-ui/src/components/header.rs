//! Header component with Portmaster core connection status and ping.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::state::AppState;

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<AppState>();
    let client = state.client.clone();
    let conn_state = client.state;
    let ping = client.ping;

    view! {
        <header class="w-full bg-slate-900/80 backdrop-blur-md border-b border-slate-800/80 px-6 py-4 sticky top-0 z-50 transition-colors">
            <div class="max-w-7xl mx-auto flex items-center justify-between gap-4 flex-wrap">
                // Left: Logo and branding
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-lg shadow-cyan-500/20 ring-1 ring-cyan-400/30">
                        <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                        </svg>
                    </div>
                    <div>
                        <div class="flex items-center space-x-2">
                            <span class="text-xl font-bold tracking-tight text-white">"Portmaster"</span>
                            <span class="px-2 py-0.5 text-xs font-semibold rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                                "Native Rust"
                            </span>
                        </div>
                        <p class="text-xs text-slate-400">"Universal Network Firewall & Privacy Suite"</p>
                    </div>
                </div>

                // Right: Core connection indicator and status badges
                <div class="flex items-center space-x-3">
                    // Connection status badge
                    <div class="flex items-center space-x-2.5 px-3 py-1.5 rounded-lg bg-slate-800/90 border border-slate-700/60 shadow-sm">
                        {move || {
                            let current = conn_state.get();
                            match current {
                                ConnectionState::Connected => view! {
                                    <div class="flex items-center space-x-2">
                                        <span class="relative flex h-2.5 w-2.5">
                                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                            <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                                        </span>
                                        <span class="text-xs font-medium text-emerald-400">"Core Online"</span>
                                        <span class="text-xs text-slate-500 font-mono">"127.0.0.1:817"</span>
                                    </div>
                                }.into_any(),
                                ConnectionState::Connecting | ConnectionState::Reconnecting => view! {
                                    <div class="flex items-center space-x-2">
                                        <span class="relative flex h-2.5 w-2.5">
                                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75"></span>
                                            <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-amber-500"></span>
                                        </span>
                                        <span class="text-xs font-medium text-amber-400">{current.label()}</span>
                                    </div>
                                }.into_any(),
                                ConnectionState::Disconnected => view! {
                                    <div class="flex items-center space-x-2">
                                        <span class="h-2.5 w-2.5 rounded-full bg-rose-500"></span>
                                        <span class="text-xs font-medium text-rose-400">"Core Offline"</span>
                                    </div>
                                }.into_any(),
                            }
                        }}

                        // Ping badge
                        {move || {
                            if let Some(ms) = ping.get() {
                                view! {
                                    <span class="pl-2 border-l border-slate-700 text-xs font-mono text-cyan-400">
                                        {format!("{} ms", ms)}
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }}
                    </div>

                    // Quick Dev-Mode badge
                    <div class="hidden sm:flex items-center px-2.5 py-1 rounded-md bg-slate-800 text-xs text-slate-300 border border-slate-700/50">
                        <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 mr-1.5"></span>
                        "PortAPI v1"
                    </div>
                </div>
            </div>
        </header>
    }
}
