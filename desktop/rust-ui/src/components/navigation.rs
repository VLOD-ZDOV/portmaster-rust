//! Left fixed icon navigation bar matching Portmaster 1:1.

use leptos::prelude::*;

use crate::client::ConnectionState;
use crate::state::{AppState, Page};

#[component]
pub fn Navigation() -> impl IntoView {
    let state = expect_context::<AppState>();
    let current_page = state.current_page;
    let is_side_dash_open = state.is_side_dash_open;
    let conn_state = state.client.state;
    let is_paused = state.is_paused;
    let pause_until = state.pause_until;

    // Popover states for Tools, Pause, Power menus
    let (is_tools_open, set_tools_open) = signal(false);
    let (is_pause_open, set_pause_open) = signal(false);
    let (is_power_open, set_power_open) = signal(false);

    let s_logo = state.clone();
    let s_arrow = state.clone();
    let s_bell = state.clone();

    view! {
        <aside class="fixed top-0 left-0 bottom-0 w-16 bg-slate-950 border-r border-slate-800/80 z-40 flex flex-col items-center justify-between py-3 select-none">
            // TOP SECTION: Logo / Lock & Indicators
            <div class="flex flex-col items-center space-y-3 w-full">
                // Animated Portmaster Logo / Security Lock
                <button
                    on:click=move |_| s_logo.toggle_side_dash()
                    title="Toggle Dashboard Drawer"
                    class="relative w-11 h-11 rounded-2xl bg-gradient-to-tr from-cyan-600 to-blue-600 flex items-center justify-center shadow-lg shadow-cyan-500/20 ring-1 ring-cyan-400/30 hover:scale-105 active:scale-95 transition-all group"
                >
                    <svg class="w-6 h-6 text-white transition-transform group-hover:rotate-45" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                    </svg>

                    // Core online status dot
                    <span class=move || {
                        match conn_state.get() {
                            ConnectionState::Connected => "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-emerald-500 ring-2 ring-slate-950 animate-pulse",
                            ConnectionState::Connecting | ConnectionState::Reconnecting => "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-amber-500 ring-2 ring-slate-950 animate-ping",
                            ConnectionState::Disconnected => "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-rose-500 ring-2 ring-slate-950",
                        }
                    }></span>
                </button>

                // Expand/collapse arrow toggle
                <button
                    on:click=move |_| s_arrow.toggle_side_dash()
                    title="Slide-out dashboard"
                    class="p-1 rounded-md text-slate-400 hover:text-white hover:bg-slate-800/80 transition-colors"
                >
                    <svg
                        class=move || {
                            if is_side_dash_open.get() {
                                "w-4 h-4 transform rotate-180 transition-transform duration-200 text-cyan-400"
                            } else {
                                "w-4 h-4 transform transition-transform duration-200 text-slate-400"
                            }
                        }
                        fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"
                    >
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                    </svg>
                </button>

                <div class="w-8 border-t border-slate-800/80 my-1"></div>

                // Notifications Bell
                <button
                    on:click=move |_| s_bell.toggle_side_dash()
                    title="Notifications"
                    class="relative p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800/80 transition-colors"
                >
                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                    </svg>
                    <span class="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-cyan-400"></span>
                </button>
            </div>

            // MIDDLE SECTION: Navigation Links
            <nav class="flex flex-col items-center space-y-2 w-full">
                // 1. Monitor / Dashboard
                {
                    let state = state.clone();
                    view! {
                        <button
                            on:click=move |_| state.navigate(Page::Monitor)
                            title="Monitor & Network Activity"
                            class=move || {
                                if current_page.get() == Page::Monitor {
                                    "p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-sm relative group"
                                } else {
                                    "p-2.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors relative group"
                                }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                            </svg>
                            <span class="absolute left-16 bg-slate-900 border border-slate-700 text-slate-200 text-xs px-2.5 py-1 rounded-md shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity whitespace-nowrap z-50">
                                "Monitor"
                            </span>
                        </button>
                    }
                }

                // 2. Applications
                {
                    let state = state.clone();
                    view! {
                        <button
                            on:click=move |_| state.navigate(Page::Apps)
                            title="Applications"
                            class=move || {
                                if matches!(current_page.get(), Page::Apps | Page::AppDetail(_)) {
                                    "p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-sm relative group"
                                } else {
                                    "p-2.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors relative group"
                                }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                            </svg>
                            <span class="absolute left-16 bg-slate-900 border border-slate-700 text-slate-200 text-xs px-2.5 py-1 rounded-md shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity whitespace-nowrap z-50">
                                "Applications"
                            </span>
                        </button>
                    }
                }

                // 3. Xray / VPN
                {
                    let state = state.clone();
                    view! {
                        <button
                            on:click=move |_| state.navigate(Page::Xray)
                            title="VPN Gateway (Xray)"
                            class=move || {
                                if current_page.get() == Page::Xray {
                                    "p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-sm relative group"
                                } else {
                                    "p-2.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors relative group"
                                }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <span class="absolute left-16 bg-slate-900 border border-slate-700 text-slate-200 text-xs px-2.5 py-1 rounded-md shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity whitespace-nowrap z-50">
                                "VPN / Xray"
                            </span>
                        </button>
                    }
                }

                // 4. Global Settings
                {
                    let state = state.clone();
                    view! {
                        <button
                            on:click=move |_| state.navigate(Page::Settings)
                            title="Global Settings"
                            class=move || {
                                if current_page.get() == Page::Settings {
                                    "p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-sm relative group"
                                } else {
                                    "p-2.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors relative group"
                                }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            </svg>
                            <span class="absolute left-16 bg-slate-900 border border-slate-700 text-slate-200 text-xs px-2.5 py-1 rounded-md shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity whitespace-nowrap z-50">
                                "Settings"
                            </span>
                        </button>
                    }
                }

                // 5. Support & Documentation
                {
                    let state = state.clone();
                    view! {
                        <button
                            on:click=move |_| state.navigate(Page::Support)
                            title="Help & Support"
                            class=move || {
                                if current_page.get() == Page::Support {
                                    "p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-sm relative group"
                                } else {
                                    "p-2.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors relative group"
                                }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <span class="absolute left-16 bg-slate-900 border border-slate-700 text-slate-200 text-xs px-2.5 py-1 rounded-md shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity whitespace-nowrap z-50">
                                "Support"
                            </span>
                        </button>
                    }
                }
            </nav>

            // BOTTOM SECTION: Tools, Pause, Power
            <div class="flex flex-col items-center space-y-2 w-full pt-2 border-t border-slate-800/80 relative">
                // 1. Tools Menu
                <div class="relative">
                    <button
                        on:click=move |_| {
                            set_tools_open.update(|v| *v = !*v);
                            set_pause_open.set(false);
                            set_power_open.set(false);
                        }
                        title="Tools & Maintenance"
                        class=move || {
                            if is_tools_open.get() {
                                "p-2 rounded-xl bg-slate-800 text-cyan-400"
                            } else {
                                "p-2 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors"
                            }
                        }
                    >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        </svg>
                    </button>

                    // Tools Popover
                    {move || {
                        if is_tools_open.get() {
                            view! {
                                <div class="absolute left-16 bottom-0 w-60 rounded-xl bg-slate-900 border border-slate-700 shadow-2xl p-2 z-50 text-xs space-y-1">
                                    <div class="px-3 py-2 border-b border-slate-800 text-slate-400">
                                        <div class="font-semibold text-white">"Portmaster Tools"</div>
                                        <div class="text-[10px] text-cyan-400">"Core v1.0.0 (Release)"</div>
                                    </div>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors">
                                        "Check for Updates"
                                    </button>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors">
                                        "Clear DNS Cache"
                                    </button>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors">
                                        "Clean up History"
                                    </button>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors">
                                        "Reload UI"
                                    </button>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors">
                                        "Copy Debug Info"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }
                    }}
                </div>

                // 2. Pause Menu
                <div class="relative">
                    <button
                        on:click=move |_| {
                            set_pause_open.update(|v| *v = !*v);
                            set_tools_open.set(false);
                            set_power_open.set(false);
                        }
                        title="Pause Protection"
                        class=move || {
                            if is_paused.get() {
                                "p-2 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/40"
                            } else if is_pause_open.get() {
                                "p-2 rounded-xl bg-slate-800 text-cyan-400"
                            } else {
                                "p-2 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors"
                            }
                        }
                    >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </button>

                    // Pause Popover
                    {
                        let state = state.clone();
                        move || {
                            if is_pause_open.get() {
                                let st = state.clone();
                                let st2 = state.clone();
                                let st3 = state.clone();
                                let st_res = state.clone();

                                view! {
                                    <div class="absolute left-16 bottom-0 w-64 rounded-xl bg-slate-900 border border-slate-700 shadow-2xl p-2 z-50 text-xs space-y-1">
                                        <div class="px-3 py-2 border-b border-slate-800">
                                            <div class="font-semibold text-white">"Pause Protection"</div>
                                            {if let Some(t) = pause_until.get() {
                                                view! {
                                                    <div class="text-[10px] text-amber-400">{t}</div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="text-[10px] text-slate-400">"Temporarily bypass filters"</div>
                                                }.into_any()
                                            }}
                                        </div>
                                        <button
                                            on:click=move |_| { st.pause_protection(5 * 60); set_pause_open.set(false); }
                                            class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
                                        >
                                            "Pause for 5 Minutes"
                                        </button>
                                        <button
                                            on:click=move |_| { st2.pause_protection(15 * 60); set_pause_open.set(false); }
                                            class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
                                        >
                                            "Pause for 15 Minutes"
                                        </button>
                                        <button
                                            on:click=move |_| { st3.pause_protection(60 * 60); set_pause_open.set(false); }
                                            class="w-full text-left px-3 py-1.5 rounded-lg text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
                                        >
                                            "Pause for 1 Hour"
                                        </button>
                                        <div class="border-t border-slate-800 my-1"></div>
                                        <button
                                            on:click=move |_| { st_res.resume_protection(); set_pause_open.set(false); }
                                            class="w-full text-left px-3 py-1.5 rounded-lg text-emerald-400 hover:bg-emerald-500/10 font-semibold transition-colors"
                                        >
                                            "Resume Protection"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }
                    }
                </div>

                // 3. Power Menu
                <div class="relative">
                    <button
                        on:click=move |_| {
                            set_power_open.update(|v| *v = !*v);
                            set_tools_open.set(false);
                            set_pause_open.set(false);
                        }
                        title="Power / Core Control"
                        class=move || {
                            if is_power_open.get() {
                                "p-2 rounded-xl bg-slate-800 text-rose-400"
                            } else {
                                "p-2 rounded-xl text-slate-400 hover:text-rose-400 hover:bg-slate-800/60 transition-colors"
                            }
                        }
                    >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                        </svg>
                    </button>

                    // Power Popover
                    {move || {
                        if is_power_open.get() {
                            view! {
                                <div class="absolute left-16 bottom-0 w-56 rounded-xl bg-slate-900 border border-slate-700 shadow-2xl p-2 z-50 text-xs space-y-1">
                                    <div class="px-3 py-2 border-b border-slate-800 font-semibold text-white">
                                        "Core Control"
                                    </div>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-amber-400 hover:bg-amber-500/10 font-medium transition-colors">
                                        "Restart Core Service"
                                    </button>
                                    <button class="w-full text-left px-3 py-1.5 rounded-lg text-rose-400 hover:bg-rose-500/10 font-medium transition-colors">
                                        "Shutdown Portmaster"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }
                    }}
                </div>
            </div>
        </aside>
    }
}
