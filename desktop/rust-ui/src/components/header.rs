//! Top header component matching Portmaster's layout and dark palette.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::state::{AppState, ViewTab};

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<AppState>();
    let client = state.client.clone();
    let conn_state = client.state;
    let ping = client.ping;
    let active_view = state.active_view;
    let xray_status = state.xray_status;
    let filter_query = state.filter_query;
    let set_filter_query = state.set_filter_query;

    view! {
        <header class="h-14 bg-[#1b1b1b] border-b border-[#2c2c2c] px-6 flex items-center justify-between gap-4 sticky top-0 z-20 flex-shrink-0">
            // Left: Current Page Title & Subtitle
            <div class="flex items-center space-x-3">
                <h1 class="text-base font-semibold text-white tracking-wide">
                    {move || match active_view.get() {
                        ViewTab::Apps => "Applications",
                        ViewTab::Vpn => "VPN (Xray)",
                        ViewTab::Settings => "Settings",
                        ViewTab::Monitor => "Network Activity",
                    }}
                </h1>
                <span class="text-xs text-[#666666]">"•"</span>
                <span class="text-xs text-[#ababab]">"Portmaster Native"</span>
            </div>

            // Center: Search input
            <div class="relative max-w-sm w-full hidden md:block">
                <input
                    type="text"
                    placeholder="Search apps, domains, settings..."
                    prop:value=move || filter_query.get()
                    on:input=move |ev| {
                        set_filter_query.set(event_target_value(&ev));
                    }
                    class="w-full pl-9 pr-3 py-1.5 rounded bg-[#121213] border border-[#2c2c2c] text-xs text-white placeholder-[#666666] focus:outline-none focus:border-[#4e97fa] transition-colors"
                />
                <svg class="w-4 h-4 text-[#666666] absolute left-2.5 top-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="7" />
                    <path stroke-linecap="round" d="M21 21l-4.35-4.35" />
                </svg>
            </div>

            // Right: VPN Status Pill & Core Status
            <div class="flex items-center space-x-3">
                // VPN Status Indicator
                {move || {
                    let s = xray_status.get();
                    if s.connected {
                        let node = s.node_name.unwrap_or_else(|| "Connected".to_string());
                        view! {
                            <div class="flex items-center space-x-2 px-2.5 py-1 rounded bg-[#18823d]/20 border border-[#1de966]/30 text-xs text-[#1de966]">
                                <span class="w-2 h-2 rounded-full bg-[#1de966] animate-pulse"></span>
                                <span class="font-medium">{format!("VPN: {}", node)}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center space-x-2 px-2.5 py-1 rounded bg-[#222222] border border-[#2c2c2c] text-xs text-[#ababab]">
                                <span class="w-2 h-2 rounded-full bg-[#666666]"></span>
                                <span>"VPN: Offline"</span>
                            </div>
                        }.into_any()
                    }
                }}

                // Core status badge
                <div class="flex items-center space-x-2 px-2.5 py-1 rounded bg-[#222222] border border-[#2c2c2c] text-xs text-[#ababab]">
                    {move || match conn_state.get() {
                        ConnectionState::Connected => view! {
                            <span class="w-2 h-2 rounded-full bg-[#1de966]"></span>
                            <span class="text-white font-medium">"Core"</span>
                        }.into_any(),
                        ConnectionState::Connecting | ConnectionState::Reconnecting => view! {
                            <span class="w-2 h-2 rounded-full bg-[#e9d81d]"></span>
                            <span class="text-[#e9d81d]">"Connecting"</span>
                        }.into_any(),
                        ConnectionState::Disconnected => view! {
                            <span class="w-2 h-2 rounded-full bg-[#e01d1d]"></span>
                            <span class="text-[#e01d1d]">"Offline"</span>
                        }.into_any(),
                    }}

                    // Ping
                    {move || {
                        if let Some(ms) = ping.get() {
                            view! {
                                <span class="pl-1.5 border-l border-[#2c2c2c] text-[11px] font-mono text-[#4e97fa]">
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
