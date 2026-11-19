//! Top header component matching Portmaster's layout and dark palette.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::state::{AppState, Page};

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<AppState>();
    let client = state.client.clone();
    let conn_state = client.state;
    let ping = client.ping;
    let current_page = state.current_page;
    let xray_status = state.xray_status;
    let filter_query = state.filter_query;
    let set_filter_query = state.set_filter_query;

    view! {
        <header class="h-12 bg-[#1b1b1b] border-b border-[#2c2c2c] px-6 flex items-center justify-between gap-4 sticky top-0 z-20 flex-shrink-0">
            // Left: Current Page Title & Subtitle
            <div class="flex items-center space-x-2.5">
                <h1 class="text-xs font-semibold text-white tracking-wide">
                    {move || match current_page.get() {
                        Page::Apps => "Applications",
                        Page::AppDetail(_) => "Application Details",
                        Page::Xray => "VPN (Xray)",
                        Page::Settings => "Settings",
                        Page::Monitor => "Network Activity",
                        Page::Support => "Documentation & Support",
                    }}
                </h1>
                <span class="text-xs text-[#666666]">"•"</span>
                <span class="text-xs text-[#888888]">"Portmaster"</span>
            </div>

            // Center: Search input
            <div class="relative max-w-xs w-full hidden md:block">
                <input
                    type="text"
                    placeholder="Search apps, domains, settings..."
                    prop:value=move || filter_query.get()
                    on:input=move |ev| {
                        set_filter_query.set(event_target_value(&ev));
                    }
                    class="input-pm !py-1 !pl-8 text-xs"
                />
                <svg class="w-3.5 h-3.5 text-[#888888] absolute left-2.5 top-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="7" />
                    <path stroke-linecap="round" d="M21 21l-4.35-4.35" />
                </svg>
            </div>

            // Right: VPN Status Pill & Core Status
            <div class="flex items-center space-x-2.5">
                // VPN Status Indicator
                {move || {
                    let s = xray_status.get();
                    if s.connected {
                        let node = s.node_name.unwrap_or_else(|| "Connected".to_string());
                        view! {
                            <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#143d24] border border-[#1de966]/30 text-[11px] text-[#1de966]">
                                <span class="w-1.5 h-1.5 rounded-full bg-[#1de966] animate-pulse"></span>
                                <span class="font-medium">{format!("VPN: {}", node)}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#222222] border border-[#2c2c2c] text-[11px] text-[#888888]">
                                <span class="w-1.5 h-1.5 rounded-full bg-[#666666]"></span>
                                <span>"VPN: Offline"</span>
                            </div>
                        }.into_any()
                    }
                }}

                // Core status badge
                <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#222222] border border-[#2c2c2c] text-[11px] text-[#888888]">
                    {move || match conn_state.get() {
                        ConnectionState::Connected => view! {
                            <span class="w-1.5 h-1.5 rounded-full bg-[#1de966]"></span>
                            <span class="text-white font-medium">"Core"</span>
                        }.into_any(),
                        ConnectionState::Connecting | ConnectionState::Reconnecting => view! {
                            <span class="w-1.5 h-1.5 rounded-full bg-[#e9d81d]"></span>
                            <span class="text-[#e9d81d]">"Connecting"</span>
                        }.into_any(),
                        ConnectionState::Disconnected => view! {
                            <span class="w-1.5 h-1.5 rounded-full bg-[#e01d1d]"></span>
                            <span class="text-[#e01d1d]">"Offline"</span>
                        }.into_any(),
                    }}

                    // Ping
                    {move || {
                        if let Some(ms) = ping.get() {
                            view! {
                                <span class="pl-1 border-l border-[#2c2c2c] text-[10px] font-mono text-[#4e97fa]">
                                    {format!("{}ms", ms)}
                                </span>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }
                    }}
                </div>
            </div>
        </header>
    }
}
