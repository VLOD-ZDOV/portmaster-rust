//! Root application view for Portmaster Native Leptos CSR.

use leptos::prelude::*;

use crate::client::PortApiClient;
use crate::components::{AppDetailView, AppSwitcher, Header, Navigation, SettingsView, SideDash, XrayPanel};
use crate::state::{AppState, Page};

#[component]
pub fn App() -> impl IntoView {
    // Initialize PortAPI client and app state
    let client = PortApiClient::new(None);
    let state = AppState::new(client);

    provide_context(state.clone());

    let current_page = state.current_page;
    let is_side_dash_open = state.is_side_dash_open;

    view! {
        <div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-cyan-500 selection:text-white">
            // Left Fixed Icon Navigation (Portmaster 1:1)
            <Navigation/>

            // Slide-Out Dashboard Drawer
            <SideDash/>

            // SideDash Backdrop Overlay (click to dismiss)
            {move || {
                if is_side_dash_open.get() {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.set_side_dash_open.set(false)
                            class="fixed inset-0 bg-black/50 backdrop-blur-sm z-20 left-16 transition-opacity"
                        ></div>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }
            }}

            // Main Content Area offset by fixed left navigation (64px / pl-16)
            <div class="pl-16 flex-1 flex flex-col min-h-screen">
                // Global Header
                <Header/>

                // Router / Main View Container
                <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
                    {move || {
                        match current_page.get() {
                            Page::Monitor => view! {
                                <div class="space-y-8">
                                    // Overview banner
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
                                                    "Universal Network Firewall & Privacy Suite. Direct WebSocket connection to PortAPI database with live state synchronization and Xray VPN gateway."
                                                </p>
                                            </div>

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
                                    </div>

                                    // Xray VPN Panel
                                    <XrayPanel/>

                                    // Quick Applications List
                                    <AppSwitcher/>
                                </div>
                            }.into_any(),

                            Page::Apps => view! {
                                <div class="space-y-6">
                                    <AppSwitcher/>
                                </div>
                            }.into_any(),

                            Page::AppDetail(app_id) => view! {
                                <AppDetailView app_id=app_id />
                            }.into_any(),

                            Page::Xray => view! {
                                <div class="space-y-6">
                                    <XrayPanel/>
                                </div>
                            }.into_any(),

                            Page::Settings => view! {
                                <SettingsView/>
                            }.into_any(),

                            Page::Support => view! {
                                <div class="p-8 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-6 text-sm">
                                    <div class="flex items-center space-x-3">
                                        <div class="w-10 h-10 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400">
                                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-lg font-bold text-white tracking-tight">"Portmaster Documentation & Support"</h2>
                                            <p class="text-xs text-slate-400">"Troubleshooting, network diagnostics, and documentation"</p>
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
                                        <a href="https://docs.safing.io/portmaster" target="_blank" class="p-4 rounded-xl bg-slate-950 border border-slate-800 hover:border-cyan-500/50 transition-colors block">
                                            <div class="font-bold text-white mb-1">"User Guide & Documentation"</div>
                                            <div class="text-xs text-slate-400">"Explore the official documentation on rules, DNS scoping, and split tunneling."</div>
                                        </a>
                                        <a href="https://github.com/safing/portmaster" target="_blank" class="p-4 rounded-xl bg-slate-950 border border-slate-800 hover:border-cyan-500/50 transition-colors block">
                                            <div class="font-bold text-white mb-1">"GitHub Source & Issue Tracker"</div>
                                            <div class="text-xs text-slate-400">"View source code, report bugs, and contribute to Portmaster development."</div>
                                        </a>
                                    </div>
                                </div>
                            }.into_any(),
                        }
                    }}
                </main>

                // Footer
                <footer class="border-t border-slate-900 bg-slate-950/80 py-6 px-4 text-center text-xs text-slate-500 mt-auto">
                    <div class="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
                        <span>"Portmaster • Free & Open Source Privacy by Safing"</span>
                        <span class="font-mono text-slate-600">"Native Rust rewrite • Leptos CSR + PortAPI WebSocket"</span>
                    </div>
                </footer>
            </div>
        </div>
    }
}
