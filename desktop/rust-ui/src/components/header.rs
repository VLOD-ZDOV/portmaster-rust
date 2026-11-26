//! Top header component matching Portmaster's layout and dark palette.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::i18n::{t, Lang};
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
    let lang = state.lang;

    view! {
        <header class="h-12 bg-[#1b1b1b] border-b border-[#2c2c2c] px-6 flex items-center justify-between gap-4 sticky top-0 z-20 flex-shrink-0">
            // Left: Current Page Title & Subtitle
            <div class="flex items-center space-x-2.5">
                <h1 class="text-xs font-semibold text-white tracking-wide">
                    {move || {
                        let l = lang.get();
                        match current_page.get() {
                            Page::Apps => t(l, "page.apps"),
                            Page::AppDetail(_) => t(l, "page.app_detail"),
                            Page::Xray => t(l, "page.xray"),
                            Page::Settings => t(l, "page.settings"),
                            Page::Monitor => t(l, "page.monitor"),
                            Page::Support => t(l, "page.support"),
                        }
                    }}
                </h1>
                <span class="text-xs text-[#666666]">"•"</span>
                <span class="text-xs text-[#888888]">"Portmaster WASM"</span>
            </div>

            // Center: Search input
            <div class="relative max-w-xs w-full hidden md:block">
                <input
                    type="text"
                    prop:placeholder=move || t(lang.get(), "header.search")
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

            // Right: Language toggle, VPN Status Pill & Core Status
            <div class="flex items-center space-x-2">
                // Compact RU / EN pill
                <button
                    type="button"
                    on:click={
                        let s = state.clone();
                        move |_| {
                            let next = if s.lang.get() == Lang::Ru { Lang::En } else { Lang::Ru };
                            s.set_language(next);
                        }
                    }
                    class="px-2 py-0.5 rounded-[2px] bg-[#222222] border border-[#2c2c2c] text-[10px] font-mono text-[#ababab] hover:text-white cursor-pointer"
                    title="Switch Language (RU/EN)"
                >
                    {move || if lang.get() == Lang::Ru { "RU" } else { "EN" }}
                </button>

                // VPN Status Indicator
                {move || {
                    let s = xray_status.get();
                    let l = lang.get();
                    if s.connected {
                        let node = s.node_name.unwrap_or_else(|| t(l, "xray.connected").to_string());
                        view! {
                            <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#143d24] border border-[#1de966]/30 text-[11px] text-[#1de966]">
                                <span class="w-1.5 h-1.5 rounded-full bg-[#1de966] animate-pulse"></span>
                                <span class="font-medium truncate max-w-[140px]">{format!("VPN: {}", node)}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#222222] border border-[#2c2c2c] text-[11px] text-[#888888]">
                                <span class="w-1.5 h-1.5 rounded-full bg-[#666666]"></span>
                                <span>{t(l, "header.vpn_offline")}</span>
                            </div>
                        }.into_any()
                    }
                }}

                // Core status badge
                <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded-[2px] bg-[#222222] border border-[#2c2c2c] text-[11px] text-[#888888]">
                    {move || {
                        let l = lang.get();
                        match conn_state.get() {
                            ConnectionState::Connected => view! {
                                <span class="w-1.5 h-1.5 rounded-full bg-[#1de966]"></span>
                                <span class="text-white font-medium">{t(l, "header.core")}</span>
                            }.into_any(),
                            ConnectionState::Connecting | ConnectionState::Reconnecting => view! {
                                <span class="w-1.5 h-1.5 rounded-full bg-[#e9d81d]"></span>
                                <span class="text-[#e9d81d]">{t(l, "header.connecting")}</span>
                            }.into_any(),
                            ConnectionState::Disconnected => view! {
                                <span class="w-1.5 h-1.5 rounded-full bg-[#e01d1d]"></span>
                                <span class="text-[#e01d1d]">{t(l, "header.offline")}</span>
                            }.into_any(),
                        }
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
