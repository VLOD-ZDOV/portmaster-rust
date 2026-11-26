//! Root application view for Portmaster Native Leptos CSR.
//!
//! Authentic 1:1 Portmaster interface without promo banners or foreign styling.

use leptos::prelude::*;

use crate::client::PortApiClient;
use crate::components::{AppDetailView, AppSwitcher, Header, MonitorPanel, Navigation, SettingsView, SideDash, XrayPanel};
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
        <div class="min-h-screen bg-[#121213] text-[#ffffff] flex flex-col font-sans select-none">
            // Left Fixed Icon Navigation (Portmaster 1:1)
            <Navigation/>

            // Slide-Out Dashboard Drawer
            <SideDash/>

            // SideDash Backdrop Overlay (click to dismiss)
            {
                let s_backdrop = state.clone();
                move || {
                    if is_side_dash_open.get() {
                        let state = s_backdrop.clone();
                        view! {
                            <div
                                on:click=move |_| state.set_side_dash_open.set(false)
                                class="fixed inset-0 bg-black/50 z-20 left-16 transition-opacity"
                            ></div>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }
            }

            // Main Content Area offset by fixed left navigation (64px / pl-16)
            <div class="pl-16 flex-1 flex flex-col min-h-screen bg-[#121213]">
                // Global Header
                <Header/>

                // Router / Main View Container
                <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-6">
                    {move || {
                        match current_page.get() {
                            Page::Monitor => view! {
                                <MonitorPanel/>
                            }.into_any(),

                            Page::Apps => view! {
                                <AppSwitcher/>
                            }.into_any(),

                            Page::AppDetail(app_id) => view! {
                                <AppDetailView app_id=app_id />
                            }.into_any(),

                            Page::Xray => view! {
                                <XrayPanel/>
                            }.into_any(),

                            Page::Settings => view! {
                                <SettingsView/>
                            }.into_any(),

                            Page::Support => view! {
                                <div class="p-6 rounded bg-[#1b1b1b] border border-[#2c2c2c] space-y-4 text-xs">
                                    <div class="flex items-center space-x-3">
                                        <div class="w-8 h-8 rounded bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-[#4e97fa]">
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-sm font-bold text-white">"Portmaster Documentation & Support"</h2>
                                            <p class="text-[11px] text-[#888888]">"Guides, knowledge base, network diagnostics and troubleshooting"</p>
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
                                        <a href="https://docs.safing.io/portmaster" target="_blank" class="p-3 rounded bg-[#222222] border border-[#2c2c2c] hover:border-[#4e97fa] transition-colors block">
                                            <div class="font-semibold text-white mb-0.5">"Official User Guide"</div>
                                            <div class="text-[11px] text-[#ababab]">"Explore rules syntax, DNS resolution scoping, and split tunneling configuration."</div>
                                        </a>
                                        <a href="https://github.com/safing/portmaster" target="_blank" class="p-3 rounded bg-[#222222] border border-[#2c2c2c] hover:border-[#4e97fa] transition-colors block">
                                            <div class="font-semibold text-white mb-0.5">"GitHub Source & Issue Tracker"</div>
                                            <div class="text-[11px] text-[#ababab]">"Report issues, inspect the core daemon and contribute to Portmaster."</div>
                                        </a>
                                    </div>
                                </div>
                            }.into_any(),
                        }
                    }}
                </main>

                // Footer
                <footer class="border-t border-[#2c2c2c] bg-[#121213] py-3 px-6 text-xs text-[#888888] mt-auto">
                    <div class="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-2">
                        <span>"Portmaster • Free & Open Source Privacy"</span>
                        {move || {
                            let port_txt = state
                                .ui_report
                                .get()
                                .map(|r| {
                                    if r.api_port_listening {
                                        format!("Rust WASM • HTTP {}", r.api_port_address)
                                    } else {
                                        "Rust WASM • Unix IPC".to_string()
                                    }
                                })
                                .unwrap_or_else(|| "Rust WASM • PortAPI v1".to_string());
                            view! {
                                <span class="font-mono text-[#666666]">{port_txt}</span>
                            }
                        }}
                    </div>
                </footer>
            </div>

            // Export modal overlay (Settings / Profile Export #388)
            {move || {
                if let Some((title, txt)) = state.sync_modal_text.get() {
                    let s_close = state.clone();
                    view! {
                        <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4">
                            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-md max-w-2xl w-full p-4 space-y-3 shadow-2xl">
                                <div class="flex items-center justify-between border-b border-[#2c2c2c] pb-2">
                                    <h3 class="text-sm font-semibold text-white">{title}</h3>
                                    <button
                                        on:click=move |_| s_close.set_sync_modal_text.set(None)
                                        class="text-xs text-[#888888] hover:text-white px-2 py-0.5 rounded bg-[#222222]"
                                    >
                                        "✕"
                                    </button>
                                </div>
                                <textarea
                                    readonly
                                    rows="14"
                                    prop:value=txt
                                    class="input-pm font-mono text-[11px] w-full select-all"
                                />
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }
            }}
        </div>
    }
}
