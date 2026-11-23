//! Left fixed icon navigation bar matching Portmaster 1:1.
//!
//! Replicates original Portmaster desktop navigation:
//! - Exact rotating SVG rings with connection status
//! - SideDash toggle arrow
//! - Notification bell with live badge
//! - Prompts triangle with yellow indicator
//! - 5 Main links: Monitor, Apps, Xray VPN, Settings, Support
//! - Lower menu triggers: Tools (wrench), Pause (pause circle), Power (power icon)

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
    let notifications = state.notifications;
    let pending_prompts = state.pending_prompts;

    // Popover states for Tools, Pause, Power menus
    let (is_tools_open, set_tools_open) = signal(false);
    let (is_pause_open, set_pause_open) = signal(false);
    let (is_power_open, set_power_open) = signal(false);

    let is_connected = move || conn_state.get() == ConnectionState::Connected;
    let has_notifications = move || !notifications.get().is_empty();
    let has_prompts = move || pending_prompts.get() > 0;

    let s_logo = state.clone();
    let s_arrow = state.clone();
    let s_bell = state.clone();

    view! {
        <aside class="fixed top-0 left-0 bottom-0 w-16 bg-[#121213] border-r border-[#2c2c2c] z-40 flex flex-col items-center justify-between py-3 select-none">
            // TOP SECTION: Logo, SideDash Toggle, Notifications, Prompts
            <div class="flex flex-col items-center w-full">
                // Rotating Portmaster Logo
                <div
                    on:click=move |_| s_logo.navigate(Page::Monitor)
                    title="Portmaster Dashboard / Network Monitor"
                    class="relative w-16 h-16 flex items-center justify-center cursor-pointer group"
                >
                    <div class="relative w-8 h-8 flex items-center justify-center">
                        // Layer 1: Spin reverse inner rings
                        <svg
                            viewBox="0 0 128 128"
                            class=move || {
                                if is_connected() {
                                    "absolute inset-0 w-8 h-8 spin reverse transition-opacity"
                                } else {
                                    "absolute inset-0 w-8 h-8 opacity-75"
                                }
                            }
                        >
                            <g fill-rule="evenodd">
                                <path
                                    shape-rendering="geometricPrecision"
                                    fill=move || if is_connected() { "#ffffff" } else { "#d12e2e" }
                                    opacity="0.8"
                                    d="M176.11 36.73l-5-8.61a41.53 41.53 0 00-14.73 57.22l8.55-5.12a31.58 31.58 0 0111.19-43.49z"
                                    transform="translate(-127.99 .01)"
                                />
                                <path
                                    shape-rendering="geometricPrecision"
                                    fill=move || if is_connected() { "#ffffff" } else { "#d12e2e" }
                                    opacity="0.8"
                                    d="M222.36 72.63a31.55 31.55 0 01-45 19.35l-4.62 8.84a41.54 41.54 0 0059.17-25.46z"
                                    transform="translate(-127.99 .01)"
                                />
                            </g>
                        </svg>

                        // Layer 2: Spin inner ring
                        <svg
                            viewBox="0 0 128 128"
                            class=move || {
                                if is_connected() {
                                    "absolute inset-0 w-8 h-8 spin transition-opacity"
                                } else {
                                    "absolute inset-0 w-8 h-8 opacity-60"
                                }
                            }
                        >
                            <g fill-rule="evenodd">
                                <path
                                    shape-rendering="geometricPrecision"
                                    fill=move || if is_connected() { "#ffffff" } else { "#d12e2e" }
                                    opacity="0.6"
                                    d="M197 83a19.66 19.66 0 01-19.25-32.57l-4.5-4.27A25.87 25.87 0 00198.59 89z"
                                    transform="translate(-127.99 .01)"
                                />
                            </g>
                        </svg>

                        // Layer 3: Outer ring
                        <svg viewBox="0 0 128 128" class="absolute inset-0 w-8 h-8">
                            <g fill-rule="evenodd">
                                <path
                                    shape-rendering="geometricPrecision"
                                    fill=move || if is_connected() { "#ffffff" } else { "#d12e2e" }
                                    d="M192 112.64A48.64 48.64 0 11240.64 64 48.64 48.64 0 01192 112.64zM256 64a64 64 0 10-64 64 64 64 0 0064-64z"
                                    transform="translate(-127.99 .1)"
                                />
                            </g>
                        </svg>
                    </div>
                </div>

                // SideDash Toggle Arrow
                <div
                    on:click=move |_| s_arrow.toggle_side_dash()
                    title="Toggle Dashboard Drawer"
                    class="w-8 h-8 my-1 flex items-center justify-center cursor-pointer rounded hover:bg-[#222222] text-[#888888] hover:text-[#ffffff] transition-colors"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        class=move || {
                            if is_side_dash_open.get() {
                                "w-4 h-4 transform rotate-180 transition-transform duration-200 text-[#4e97fa]"
                            } else {
                                "w-4 h-4 transform transition-transform duration-200 text-[#888888]"
                            }
                        }
                    >
                        <path
                            fill-rule="evenodd"
                            d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                            clip-rule="evenodd"
                        />
                    </svg>
                </div>

                <div class="w-8 border-t border-[#2c2c2c] my-1"></div>

                // Notifications Bell Icon
                <div
                    on:click=move |_| s_bell.toggle_side_dash()
                    title="Notifications"
                    class="relative w-8 h-8 my-1 flex items-center justify-center cursor-pointer rounded hover:bg-[#222222] text-[#888888] hover:text-[#ffffff] transition-colors"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                        class="w-4 h-4"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                        />
                    </svg>
                    {move || if has_notifications() {
                        view! {
                            <span class="absolute top-1.5 right-1.5 w-1.5 h-1.5 bg-[#4e97fa] rounded-full"></span>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }}
                </div>

                // Prompts Triangle Icon
                {
                    let state = state.clone();
                    move || if has_prompts() {
                        let s_p = state.clone();
                        view! {
                            <div
                                on:click=move |_| s_p.navigate(Page::Monitor)
                                title="Pending connection decisions"
                                class="relative w-8 h-8 my-1 flex items-center justify-center cursor-pointer rounded hover:bg-[#222222] text-[#e9d31d] transition-colors"
                            >
                                <span class="absolute top-1.5 right-1.5 w-1.5 h-1.5 bg-[#e9d31d] rounded-full animate-ping"></span>
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }
            </div>

            // MIDDLE SECTION: 5 Main Navigation Links matching Portmaster 1:1
            <nav class="flex flex-col items-center w-full">
                // 1. Network Activity (Monitor)
                {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.navigate(Page::Monitor)
                            title="Network Activity & Monitor"
                            class=move || {
                                if current_page.get() == Page::Monitor {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer bg-[#2c2c2c] text-white transition-all scale-105"
                                } else {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M8.464 8.464c-1.953 1.953-1.953 5.118 0 7.071 1.953 1.953 5.118 1.953 7.071 0 1.953-1.953 1.953-5.119 0-7.071C14.559 7.488 13.28 7 12 7" />
                                <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636c-3.515 3.515-3.515 9.213 0 12.728 3.515 3.515 9.213 3.515 12.728 0 3.515-3.515 3.515-9.213 0-12.728-2.627-2.627-6.474-3.289-9.717-1.989M5.64 5.64L12 12" />
                            </svg>
                        </div>
                    }
                }

                // 2. Applications
                {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.navigate(Page::Apps)
                            title="Applications"
                            class=move || {
                                let is_active = matches!(
                                    current_page.get(),
                                    Page::Apps | Page::AppDetail(_)
                                );
                                if is_active {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer bg-[#2c2c2c] text-white transition-all scale-105"
                                } else {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" fill="currentColor" d="M19 21h-3a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h3a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2Z" />
                                <path fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9h-3a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2ZM5 3h3a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2ZM5 15h3a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-2a2 2 0 0 1 2-2Z" />
                            </svg>
                        </div>
                    }
                }

                // 3. Xray VPN (SPN / Map)
                {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.navigate(Page::Xray)
                            title="Xray VPN Gateway"
                            class=move || {
                                if current_page.get() == Page::Xray {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer bg-[#2c2c2c] text-white transition-all scale-105"
                                } else {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="2">
                                <g stroke-linecap="round" stroke-linejoin="round">
                                    <circle cx="12" cy="12" r="9" />
                                    <path d="M3 12h18" />
                                    <path d="M12 3c2.5 2.5 3.8 5.5 3.8 9S14.5 18.5 12 21c-2.5-2.5-3.8-5.5-3.8-9S9.5 5.5 12 3z" />
                                </g>
                            </svg>
                        </div>
                    }
                }

                // 4. Global Settings
                {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.navigate(Page::Settings)
                            title="Global Settings"
                            class=move || {
                                if current_page.get() == Page::Settings {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer bg-[#2c2c2c] text-white transition-all scale-105"
                                } else {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="2">
                                <g stroke-linecap="round" stroke-linejoin="round">
                                    <path shape-rendering="geometricPrecision" d="M13.7678 10.2322c.976311.976311.976311 2.55922 0 3.53553-.976311.976311-2.55922.976311-3.53553 0-.976311-.976311-.976311-2.55922 0-3.53553.976311-.976311 2.55922-.976311 3.53553 0" />
                                    <path shape-rendering="geometricPrecision" d="M14.849 4.12l.583.194c.534.178.895.678.895 1.241v.837c0 .712.568 1.293 1.28 1.308l.838.018c.485.01.925.289 1.142.723l.275.55c.252.504.153 1.112-.245 1.51l-.592.592c-.503.503-.512 1.316-.02 1.83l.58.606c.336.351.45.858.296 1.319l-.194.583c-.178.534-.678.895-1.241.895h-.837c-.712 0-1.293.568-1.308 1.28l-.018.838c-.01.485-.289.925-.723 1.142l-.55.275c-.504.252-1.112.153-1.51-.245l-.592-.592c-.503-.503-1.316-.512-1.83-.02l-.606.58c-.351.336-.858.45-1.319.296l-.583-.194c-.534-.178-.895-.678-.895-1.241v-.837c0-.712-.568-1.293-1.28-1.308l-.838-.018c-.485-.01-.925-.289-1.142-.723l-.275-.55c-.252-.504-.153-1.112.245-1.51l.592-.592c.503-.503.512-1.316.02-1.83l-.58-.606c-.337-.352-.451-.86-.297-1.32l.194-.583c.178-.534.678-.895 1.241-.895h.837c.712 0 1.293-.568 1.308-1.28l.018-.838c.012-.485.29-.925.724-1.142l.55-.275c.504-.252 1.112-.153 1.51.245l.592.592c.503.503 1.316.512 1.83.02l.606-.58c.351-.335.859-.449 1.319-.295z" />
                                </g>
                            </svg>
                        </div>
                    }
                }

                <div class="w-8 border-t border-[#2c2c2c] my-1"></div>

                // 5. Support / Help
                {
                    let state = state.clone();
                    view! {
                        <div
                            on:click=move |_| state.navigate(Page::Support)
                            title="Documentation & Support"
                            class=move || {
                                if current_page.get() == Page::Support {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer bg-[#2c2c2c] text-white transition-all scale-105"
                                } else {
                                    "w-8 h-8 my-2 rounded-[10px] flex items-center justify-center cursor-pointer text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="2">
                                <g stroke-linecap="round" stroke-linejoin="round">
                                    <path shape-rendering="geometricPrecision" d="M12 21v0c-4.971 0-9-4.029-9-9v0c0-4.971 4.029-9 9-9v0c4.971 0 9 4.029 9 9v0c0 4.971-4.029 9-9 9z" />
                                    <path shape-rendering="geometricPrecision" d="M12 13.25V13c0-.817.505-1.26 1.011-1.6.494-.333.989-.767.989-1.567 0-1.105-.895-2-2-2s-2 .895-2 2M11.999 16c-.138 0-.25.112-.249.25 0 .138.112.25.25.25s.25-.112.25-.25-.112-.25-.251-.25" />
                                </g>
                            </svg>
                        </div>
                    }
                }
            </nav>

            // LOWER SECTION: Tools, Pause, Power Menus
            <div class="flex flex-col items-center w-full space-y-1 pb-2 relative">
                // 1. Tools Menu (Wrench)
                <div class="relative">
                    <button
                        on:click=move |_| set_tools_open.update(|v| *v = !*v)
                        title="Tools & Maintenance"
                        class=move || {
                            if is_tools_open.get() {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center bg-[#2c2c2c] text-white transition-all"
                            } else {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="1.5">
                            <g stroke-linecap="round" stroke-linejoin="round">
                                <path d="M11.835,15l5,5c0.828,0.828 2.172,0.828 3,0v0c0.828,-0.828 0.828,-2.172 0,-3l-5,-5"></path>
                                <path d="M20.916,5.847c0.024,0.023 0.042,0.053 0.051,0.085c0.47,1.567 0.106,3.33 -1.132,4.568c-1.251,1.251 -3.038,1.609 -4.617,1.117l-8.347,8.347c-0.813,0.813 -2.139,0.874 -2.98,0.09c-0.884,-0.823 -0.902,-2.207 -0.056,-3.054l8.383,-8.383c-0.492,-1.579 -0.134,-3.366 1.117,-4.617c1.238,-1.238 3.001,-1.602 4.568,-1.132c0.032,0.01 0.062,0.027 0.085,0.051l0.162,0.162c0.078,0.078 0.078,0.205 0,0.283l-2.635,2.636l2.32,2.32l2.636,-2.636c0.078,-0.078 0.205,-0.078 0.283,0l0.162,0.163Z"></path>
                                <path d="M2.933,4.293l0.674,2.023c0.136,0.409 0.518,0.684 0.949,0.684h2.279v-2.279c0,-0.43 -0.275,-0.813 -0.684,-0.949l-2.023,-0.674c-0.18,-0.06 -0.378,-0.013 -0.512,0.121l-0.562,0.562c-0.134,0.134 -0.181,0.332 -0.121,0.512Z"></path>
                                <path d="M6.84,7l3.5,3.5"></path>
                            </g>
                        </svg>
                    </button>

                    // Tools Popover Menu
                    {
                        let state = state.clone();
                        move || {
                            if is_tools_open.get() {
                                let s_tools = state.clone();
                                let s_reinit = state.clone();
                                let s_dns = state.clone();
                                let s_cleanup = state.clone();
                                view! {
                                    <div class="fixed left-16 bottom-14 w-60 bg-[#1b1b1b] border border-[#2c2c2c] rounded-md shadow-2xl py-2 z-50 text-xs">
                                        <div class="px-3 py-1.5 border-b border-[#2c2c2c] text-[11px] text-[#ababab]">
                                            <div>"Core: "<span class="text-white font-mono">{concat!("v", env!("CARGO_PKG_VERSION"))}</span></div>
                                            <div>"Channel: "<span class="text-white font-mono uppercase">"stable"</span></div>
                                        </div>
                                        <button on:click=move |_| {
                                            s_tools.check_updates();
                                            set_tools_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"Check Updates"</span>
                                        </button>
                                        <button on:click=move |_| set_tools_open.set(false) class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"View Changelog"</span>
                                        </button>
                                        <div class="border-t border-[#2c2c2c] my-1"></div>
                                        <button on:click=move |_| {
                                            if let Some(win) = web_sys::window() {
                                                let _ = win.location().reload();
                                            }
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"Reload UI"</span>
                                        </button>
                                        <button on:click=move |_| {
                                            s_reinit.restart_xray();
                                            set_tools_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"Restart Xray VPN"</span>
                                        </button>
                                        <button on:click=move |_| {
                                            s_dns.clear_dns_cache();
                                            set_tools_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"Clear DNS Cache"</span>
                                        </button>
                                        <button on:click=move |_| {
                                            s_cleanup.clear_history();
                                            set_tools_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white flex items-center space-x-2">
                                            <span>"Cleanup History"</span>
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }
                    }
                </div>

                // 2. Pause Menu (Circle Pause)
                <div class="relative">
                    <button
                        on:click=move |_| set_pause_open.update(|v| *v = !*v)
                        title="Pause Protection"
                        class=move || {
                            if is_pause_open.get() || is_paused.get() {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center bg-[#2c2c2c] text-white transition-all"
                            } else {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="1.5">
                            <g stroke-linecap="round" stroke-linejoin="round">
                                <path d="M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" />
                                <path d="M14 9L14 15" />
                                <path d="M10 9L10 15" />
                            </g>
                        </svg>
                    </button>

                    // Pause Popover Menu
                    {
                        let state = state.clone();
                        move || {
                            if is_pause_open.get() {
                                let s1 = state.clone();
                                let s2 = state.clone();
                                let s3 = state.clone();
                                let s4 = state.clone();
                                let s5 = state.clone();
                                let s6 = state.clone();
                                let sr = state.clone();
                                view! {
                                    <div class="fixed left-16 bottom-8 w-56 bg-[#1b1b1b] border border-[#2c2c2c] rounded-md shadow-2xl py-2 z-50 text-xs">
                                        <div class="px-3 py-1 text-[11px] text-[#ababab] font-medium border-b border-[#2c2c2c] mb-1">
                                            "Pause VPN / Xray"
                                        </div>
                                        <button on:click=move |_| {
                                            s1.pause(5, true);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause VPN for 5 Minutes"
                                        </button>
                                        <button on:click=move |_| {
                                            s2.pause(15, true);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause VPN for 15 Minutes"
                                        </button>
                                        <button on:click=move |_| {
                                            s3.pause(60, true);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause VPN for 1 Hour"
                                        </button>

                                        <div class="border-t border-[#2c2c2c] my-1"></div>
                                        <div class="px-3 py-1 text-[11px] text-[#ababab] font-medium">
                                            "Pause All Protection"
                                        </div>
                                        <button on:click=move |_| {
                                            s4.pause(5, false);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause All for 5 Minutes"
                                        </button>
                                        <button on:click=move |_| {
                                            s5.pause(15, false);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause All for 15 Minutes"
                                        </button>
                                        <button on:click=move |_| {
                                            s6.pause(60, false);
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-white">
                                            "Pause All for 1 Hour"
                                        </button>

                                        <div class="border-t border-[#2c2c2c] my-1"></div>
                                        <button on:click=move |_| {
                                            sr.resume();
                                            set_pause_open.set(false);
                                        } class="w-full text-left px-3 py-1.5 hover:bg-[#222222] text-[#1de966] font-medium">
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
                        on:click=move |_| set_power_open.update(|v| *v = !*v)
                        title="Service Control & Power"
                        class=move || {
                            if is_power_open.get() {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center bg-[#2c2c2c] text-white transition-all"
                            } else {
                                "w-8 h-8 rounded-[10px] flex items-center justify-center text-[#ababab] opacity-50 hover:opacity-100 hover:text-white transition-all"
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24" class="w-4 h-4 text-white stroke-current fill-none" stroke-width="1.5">
                            <g stroke-linecap="round" stroke-linejoin="round">
                                <path d="M18.364,5.63604c3.51472,3.51472 3.51472,9.2132 0,12.7279c-3.51472,3.51472 -9.2132,3.51472 -12.7279,0c-3.51472,-3.51472 -3.51472,-9.2132 -1.77636e-15,-12.7279c3.51472,-3.51472 9.2132,-3.51472 12.7279,-1.77636e-15" />
                                <path d="M12,7v5" />
                                <path d="M15.534,8.466c1.952,1.952 1.952,5.117 0,7.069c-1.952,1.952 -5.117,1.952 -7.069,0c-1.952,-1.952 -1.952,-5.117 0,-7.069" />
                            </g>
                        </svg>
                    </button>

                    // Power Popover Menu
                    {
                        let s_pow1 = state.clone();
                        let s_pow2 = state.clone();
                        move || {
                            if is_power_open.get() {
                                let s1 = s_pow1.clone();
                                let s2 = s_pow2.clone();
                                view! {
                                    <div class="fixed left-16 bottom-2 w-48 bg-[#1b1b1b] border border-[#2c2c2c] rounded-md shadow-2xl py-2 z-50 text-xs">
                                        <button on:click=move |_| {
                                            s1.restart_core();
                                            set_power_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-white">
                                            "Restart Core Service"
                                        </button>
                                        <button on:click=move |_| {
                                            s2.shutdown_core();
                                            set_power_open.set(false);
                                        } class="w-full text-left px-3 py-2 hover:bg-[#222222] text-[#e01d1d]">
                                            "Shutdown Portmaster"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }
                    }
                </div>
            </div>
        </aside>
    }
}
