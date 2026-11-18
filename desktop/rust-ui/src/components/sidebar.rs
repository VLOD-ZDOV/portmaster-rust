//! Left navigation sidebar matching Portmaster's native layout.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::state::{AppState, ViewTab};

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = expect_context::<AppState>();
    let active_view = state.active_view;
    let set_active_view = state.set_active_view;
    let client = state.client.clone();
    let conn_state = client.state;

    view! {
        <aside class="w-16 flex-shrink-0 bg-[#1b1b1b] border-r border-[#2c2c2c] flex flex-col items-center justify-between py-4 select-none z-30">
            // Top: Portmaster Rotating Logo & Nav Links
            <div class="flex flex-col items-center space-y-6 w-full">
                // Portmaster Logo (Authentic Safing shield geometry)
                <div class="relative w-10 h-10 flex items-center justify-center cursor-pointer group"
                    title="Portmaster"
                    on:click=move |_| set_active_view.set(ViewTab::Apps)>
                    <svg viewBox="0 0 128 128" class="w-9 h-9 text-[#4e97fa] transition-transform duration-500 group-hover:scale-105">
                        <g fill="currentColor" fill-rule="evenodd">
                            <path opacity="0.9" d="M64 8a56 56 0 1056 56A56.06 56.06 0 0064 8zm0 100a44 44 0 1144-44 44.05 44.05 0 01-44 44z" />
                            <path opacity="0.75" d="M48 40l-5-8.6a41.5 41.5 0 00-14.7 57.2l8.5-5.1A31.6 31.6 0 0148 40z" />
                            <path opacity="0.6" d="M94.4 72.6a31.5 31.5 0 01-45 19.4l-4.6 8.8a41.5 41.5 0 0059.2-25.5z" />
                            <circle cx="64" cy="64" r="14" fill="#1de966" opacity="0.85"/>
                        </g>
                    </svg>
                </div>

                // Nav item divider
                <div class="w-8 h-px bg-[#2c2c2c]"></div>

                // Navigation Items
                <nav class="flex flex-col items-center space-y-2 w-full">
                    // 1. Network Activity / Monitor
                    <button
                        title="Network Activity"
                        on:click=move |_| set_active_view.set(ViewTab::Monitor)
                        class=move || {
                            let is_active = active_view.get() == ViewTab::Monitor;
                            if is_active {
                                "relative w-12 h-12 rounded-lg bg-[#222222] text-[#4e97fa] flex items-center justify-center transition-all shadow-sm"
                            } else {
                                "relative w-12 h-12 rounded-lg text-[#666666] hover:text-[#ababab] hover:bg-[#222222]/60 flex items-center justify-center transition-colors"
                            }
                        }
                    >
                        {move || if active_view.get() == ViewTab::Monitor {
                            view! { <span class="absolute left-0 top-2 bottom-2 w-1 bg-[#4e97fa] rounded-r"></span> }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                        // Radar / Monitor icon
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M8.464 8.464c-1.953 1.953-1.953 5.118 0 7.071 1.953 1.953 5.118 1.953 7.071 0 1.953-1.953 1.953-5.119 0-7.071C14.559 7.488 13.28 7 12 7" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636c-3.515 3.515-3.515 9.213 0 12.728 3.515 3.515 9.213 3.515 12.728 0 3.515-3.515 3.515-9.213 0-12.728-2.627-2.627-6.474-3.289-9.717-1.989M5.64 5.64L12 12" />
                        </svg>
                    </button>

                    // 2. Applications
                    <button
                        title="Applications"
                        on:click=move |_| set_active_view.set(ViewTab::Apps)
                        class=move || {
                            let is_active = active_view.get() == ViewTab::Apps;
                            if is_active {
                                "relative w-12 h-12 rounded-lg bg-[#222222] text-[#4e97fa] flex items-center justify-center transition-all shadow-sm"
                            } else {
                                "relative w-12 h-12 rounded-lg text-[#666666] hover:text-[#ababab] hover:bg-[#222222]/60 flex items-center justify-center transition-colors"
                            }
                        }
                    >
                        {move || if active_view.get() == ViewTab::Apps {
                            view! { <span class="absolute left-0 top-2 bottom-2 w-1 bg-[#4e97fa] rounded-r"></span> }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                        // 4-Squares App Grid icon
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <rect x="3" y="3" width="7" height="7" rx="1.5" />
                            <rect x="14" y="3" width="7" height="7" rx="1.5" />
                            <rect x="3" y="14" width="7" height="7" rx="1.5" />
                            <rect x="14" y="14" width="7" height="7" rx="1.5" />
                        </svg>
                    </button>

                    // 3. VPN (Xray)
                    <button
                        title="VPN (Xray)"
                        on:click=move |_| set_active_view.set(ViewTab::Vpn)
                        class=move || {
                            let is_active = active_view.get() == ViewTab::Vpn;
                            if is_active {
                                "relative w-12 h-12 rounded-lg bg-[#222222] text-[#4e97fa] flex items-center justify-center transition-all shadow-sm"
                            } else {
                                "relative w-12 h-12 rounded-lg text-[#666666] hover:text-[#ababab] hover:bg-[#222222]/60 flex items-center justify-center transition-colors"
                            }
                        }
                    >
                        {move || if active_view.get() == ViewTab::Vpn {
                            view! { <span class="absolute left-0 top-2 bottom-2 w-1 bg-[#4e97fa] rounded-r"></span> }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                        // Globe / Network icon
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M3.6 9h16.8M3.6 15h16.8" />
                            <path d="M12 3a15 15 0 014 9 15 15 0 01-4 9 15 15 0 01-4-9 15 15 0 014-9z" />
                        </svg>
                    </button>

                    // 4. Settings
                    <button
                        title="Settings"
                        on:click=move |_| set_active_view.set(ViewTab::Settings)
                        class=move || {
                            let is_active = active_view.get() == ViewTab::Settings;
                            if is_active {
                                "relative w-12 h-12 rounded-lg bg-[#222222] text-[#4e97fa] flex items-center justify-center transition-all shadow-sm"
                            } else {
                                "relative w-12 h-12 rounded-lg text-[#666666] hover:text-[#ababab] hover:bg-[#222222]/60 flex items-center justify-center transition-colors"
                            }
                        }
                    >
                        {move || if active_view.get() == ViewTab::Settings {
                            view! { <span class="absolute left-0 top-2 bottom-2 w-1 bg-[#4e97fa] rounded-r"></span> }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                        // Cog / Gear icon
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                    </button>
                </nav>
            </div>

            // Bottom: Online connection status indicator
            <div class="flex flex-col items-center space-y-3">
                <div class="w-8 h-px bg-[#2c2c2c]"></div>
                <div class="flex items-center justify-center p-2 rounded-lg cursor-pointer group"
                    title=move || format!("PortAPI: {}", conn_state.get().label())>
                    {move || match conn_state.get() {
                        ConnectionState::Connected => view! {
                            <span class="relative flex h-3 w-3">
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#1de966] opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-3 w-3 bg-[#1de966]"></span>
                            </span>
                        }.into_any(),
                        ConnectionState::Connecting | ConnectionState::Reconnecting => view! {
                            <span class="relative flex h-3 w-3">
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#e9d81d] opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-3 w-3 bg-[#e9d81d]"></span>
                            </span>
                        }.into_any(),
                        ConnectionState::Disconnected => view! {
                            <span class="h-3 w-3 rounded-full bg-[#e01d1d]"></span>
                        }.into_any(),
                    }}
                </div>
                <span class="text-[10px] text-[#666666] font-mono">"817"</span>
            </div>
        </aside>
    }
}
