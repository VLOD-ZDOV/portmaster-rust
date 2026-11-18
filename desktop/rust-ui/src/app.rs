//! Root application view for Portmaster Native Leptos CSR.

use leptos::prelude::*;

use crate::client::PortApiClient;
use crate::components::{AppSwitcher, Header, XrayPanel};
use crate::state::AppState;

#[component]
pub fn App() -> impl IntoView {
    // Initialize PortAPI client and app state
    let client = PortApiClient::new(None);
    let state = AppState::new(client);

    provide_context(state);

    view! {
        <div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-cyan-500 selection:text-white">
            // Global Header
            <Header/>

            // Main Content Area
            <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
                // Welcome / Overview banner
                <div class="relative overflow-hidden rounded-2xl bg-gradient-to-r from-slate-900 via-indigo-950/40 to-slate-900 border border-slate-800 p-6 sm:p-8 shadow-xl">
                    <div class="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
                        <div>
                            <div class="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 text-xs font-semibold mb-3">
                                <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
                                <span>"Native Rust Core & Leptos UI"</span>
                            </div>
                            <h1 class="text-2xl sm:text-3xl font-extrabold text-white tracking-tight">
                                "Portmaster Control Center"
                            </h1>
                            <p class="mt-2 text-sm text-slate-400 max-w-2xl leading-relaxed">
                                "Zero-overhead native Rust interface powered by Leptos CSR & Trunk. Direct WebSocket connection to PortAPI database with live state synchronization and Xray VPN gateway."
                            </p>
                        </div>

                        // Quick info cards
                        <div class="flex items-center space-x-3 sm:space-x-4">
                            <div class="px-4 py-3 rounded-xl bg-slate-950/80 border border-slate-800/80 text-center min-w-[100px]">
                                <span class="text-xs text-slate-400 block font-medium">"Architecture"</span>
                                <span class="text-sm font-bold text-cyan-400 font-mono">"WASM / CSR"</span>
                            </div>
                            <div class="px-4 py-3 rounded-xl bg-slate-950/80 border border-slate-800/80 text-center min-w-[100px]">
                                <span class="text-xs text-slate-400 block font-medium">"Styling"</span>
                                <span class="text-sm font-bold text-cyan-400 font-mono">"TailwindCSS"</span>
                            </div>
                        </div>
                    </div>

                    // Decorative glow
                    <div class="absolute -right-16 -top-16 w-64 h-64 rounded-full bg-cyan-500/10 blur-3xl pointer-events-none"></div>
                    <div class="absolute -left-16 -bottom-16 w-64 h-64 rounded-full bg-indigo-500/10 blur-3xl pointer-events-none"></div>
                </div>

                // Section 1: VPN Gateway (Xray) Control Panel
                <XrayPanel/>

                // Section 2: Application Switcher & Router
                <AppSwitcher/>
            </main>

            // Footer
            <footer class="border-t border-slate-900 bg-slate-950/80 py-6 px-4 text-center text-xs text-slate-500">
                <div class="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
                    <span>"Portmaster • Free & Open Source Privacy by Safing"</span>
                    <span class="font-mono text-slate-600">"Native Rust rewrite • Leptos CSR + PortAPI WebSocket"</span>
                </div>
            </footer>
        </div>
    }
}
