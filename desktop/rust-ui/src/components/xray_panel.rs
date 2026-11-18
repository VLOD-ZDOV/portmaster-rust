//! VPN (Xray) control panel styled 1:1 with Portmaster Xray page.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn XrayPanel() -> impl IntoView {
    let state = expect_context::<AppState>();
    let xray_status = state.xray_status;
    let xray_nodes = state.xray_nodes;
    let is_busy = state.is_busy;
    let route_by_default = state.route_by_default;

    let (new_sub_url, set_new_sub_url) = signal(String::new());

    let on_toggle_vpn = {
        let state = state.clone();
        move |_| {
            state.toggle_vpn();
        }
    };

    let on_toggle_default_route = {
        let state = state.clone();
        move |_| {
            state.toggle_route_by_default();
        }
    };

    view! {
        <div class="space-y-6">
            // 1. Connection Status Banner Card
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-5">
                <div class="flex items-center justify-between flex-wrap gap-4">
                    <div class="flex items-center space-x-4">
                        // Status Icon
                        <div class=move || {
                            let s = xray_status.get();
                            if s.connected {
                                "w-12 h-12 rounded-lg bg-[#18823d]/20 border border-[#1de966]/40 flex items-center justify-center text-[#1de966]"
                            } else {
                                "w-12 h-12 rounded-lg bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-[#666666]"
                            }
                        }>
                            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <circle cx="12" cy="12" r="9" />
                                <path d="M3.6 9h16.8M3.6 15h16.8" />
                                <path d="M12 3a15 15 0 014 9 15 15 0 01-4 9 15 15 0 01-4-9 15 15 0 014-9z" />
                            </svg>
                        </div>

                        <div>
                            <div class="flex items-center space-x-2.5">
                                <h2 class="text-base font-semibold text-white">
                                    {move || {
                                        let s = xray_status.get();
                                        if s.connected {
                                            s.node_name.unwrap_or_else(|| "Connected to VPN".to_string())
                                        } else {
                                            "VPN Disconnected".to_string()
                                        }
                                    }}
                                </h2>
                                {move || {
                                    let s = xray_status.get();
                                    if s.connected {
                                        view! {
                                            <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-[#18823d]/20 text-[#1de966] border border-[#1de966]/30">
                                                "Active"
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-[#222222] text-[#666666] border border-[#2c2c2c]">
                                                "Offline"
                                            </span>
                                        }.into_any()
                                    }
                                }}
                            </div>

                            <p class="text-xs text-[#ababab] mt-1 flex items-center space-x-2">
                                {move || {
                                    let s = xray_status.get();
                                    if s.connected {
                                        let ip = s.exit_ip.unwrap_or_else(|| "Unknown".to_string());
                                        let ctry = s.exit_country.unwrap_or_else(|| "NL".to_string());
                                        format!("Exit IP: {} ({}) • Encrypted tunnel active", ip, ctry)
                                    } else {
                                        "Traffic is routed directly without encryption bypass.".to_string()
                                    }
                                }}
                            </p>
                        </div>
                    </div>

                    // Connect / Disconnect button
                    <button
                        on:click=on_toggle_vpn
                        disabled=move || is_busy.get()
                        class=move || {
                            let s = xray_status.get();
                            if is_busy.get() {
                                "px-4 py-2 rounded text-xs font-medium bg-[#222222] text-[#666666] cursor-not-allowed border border-[#2c2c2c]"
                            } else if s.connected {
                                "px-4 py-2 rounded text-xs font-medium bg-[#e01d1d] hover:bg-[#d12e2e] text-white transition-colors shadow-sm"
                            } else {
                                "px-4 py-2 rounded text-xs font-medium bg-[#4e97fa] hover:bg-[#3d86e8] text-white transition-colors shadow-sm"
                            }
                        }
                    >
                        {move || {
                            let s = xray_status.get();
                            if is_busy.get() {
                                "Working..."
                            } else if s.connected {
                                "Disconnect"
                            } else {
                                "Connect VPN"
                            }
                        }}
                    </button>
                </div>
            </div>

            // 2. Default Routing Global Setting (spn/use)
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-4 flex items-center justify-between gap-4">
                <div class="flex items-center space-x-3">
                    // Big Toggle
                    <button
                        type="button"
                        on:click=on_toggle_default_route
                        class=move || {
                            if route_by_default.get() {
                                "w-11 h-6 rounded-full bg-[#4e97fa] p-0.5 transition-colors cursor-pointer flex items-center"
                            } else {
                                "w-11 h-6 rounded-full bg-[#2c2c2c] p-0.5 transition-colors cursor-pointer flex items-center"
                            }
                        }
                    >
                        <span class=move || {
                            if route_by_default.get() {
                                "w-5 h-5 rounded-full bg-white shadow-sm transform translate-x-5 transition-transform"
                            } else {
                                "w-5 h-5 rounded-full bg-[#ababab] shadow-sm transform translate-x-0 transition-transform"
                            }
                        }></span>
                    </button>

                    <div>
                        <div class="text-sm font-semibold text-white">
                            "Route Through VPN (Xray) by Default"
                        </div>
                        <div class="text-xs text-[#ababab]">
                            "Direct all application network traffic through VPN (Xray). When OFF, traffic connects directly unless an app is explicitly enabled below or matched by Xray routing rules."
                        </div>
                    </div>
                </div>

                <div class="text-right flex-shrink-0">
                    {move || {
                        if route_by_default.get() {
                            view! {
                                <span class="px-2 py-0.5 rounded text-xs font-semibold bg-[#4e97fa]/15 text-[#4e97fa] border border-[#4e97fa]/30">
                                    "Default: ON"
                                </span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="px-2 py-0.5 rounded text-xs font-semibold bg-[#222222] text-[#ababab] border border-[#2c2c2c]">
                                    "Default: OFF"
                                </span>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // 3. Subscriptions Section
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-4 space-y-3">
                <div class="flex items-center justify-between">
                    <h3 class="text-sm font-semibold text-white">"Subscriptions & Keys"</h3>
                    <span class="text-xs text-[#666666]">"VLESS, VMess, Trojan, WireGuard"</span>
                </div>

                <div class="flex items-center space-x-2">
                    <input
                        type="text"
                        placeholder="Paste subscription URL, vless://, wireguard:// or base64 key..."
                        prop:value=move || new_sub_url.get()
                        on:input=move |ev| {
                            set_new_sub_url.set(event_target_value(&ev));
                        }
                        class="flex-1 bg-[#121213] border border-[#2c2c2c] rounded px-3 py-1.5 text-xs text-white placeholder-[#666666] focus:border-[#4e97fa] focus:outline-none"
                    />
                    <button
                        on:click=move |_| {
                            let url = new_sub_url.get();
                            if !url.is_empty() {
                                set_new_sub_url.set(String::new());
                            }
                        }
                        class="px-3 py-1.5 rounded text-xs font-medium bg-[#4e97fa] hover:bg-[#3d86e8] text-white transition-colors"
                    >
                        "Add"
                    </button>
                </div>
            </div>

            // 4. Server Nodes Selector
            <div class="space-y-3">
                <div class="flex items-center justify-between pb-1 border-b border-[#2c2c2c]">
                    <h3 class="text-sm font-semibold text-white">
                        {move || format!("Server Nodes ({})", xray_nodes.get().len())}
                    </h3>
                    <span class="text-xs text-[#666666]">"Click a node to connect immediately"</span>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {move || {
                        let current_status = xray_status.get();
                        let current_id = current_status.node_id.clone();
                        let nodes = xray_nodes.get();

                        nodes.into_iter().map(|node| {
                            let node_id = node.id.clone();
                            let is_active = current_id.as_deref() == Some(&node_id) && current_status.connected;

                            let state_clone = state.clone();
                            let node_id_connect = node.id.clone();
                            let on_select = move |_| {
                                state_clone.select_node(node_id_connect.clone());
                            };

                            view! {
                                <div class=if is_active {
                                    "bg-[#1b1b1b] border border-[#4e97fa] rounded-lg p-3 flex items-center justify-between gap-3 shadow-sm transition-all"
                                } else {
                                    "bg-[#1b1b1b] border border-[#2c2c2c] hover:border-[#3a3a3a] rounded-lg p-3 flex items-center justify-between gap-3 transition-all"
                                }>
                                    <div class="flex items-center space-x-3 min-w-0">
                                        <div class="w-8 h-8 rounded bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-xs font-bold text-white flex-shrink-0">
                                            {node.country_code.clone()}
                                        </div>
                                        <div class="min-w-0">
                                            <div class="flex items-center space-x-2">
                                                <span class="text-xs font-medium text-white truncate">{node.name.clone()}</span>
                                                <span class="px-1.5 py-0.2 rounded text-[10px] font-mono bg-[#222222] text-[#ababab] border border-[#2c2c2c] uppercase">
                                                    {node.protocol.clone()}
                                                </span>
                                            </div>
                                            <p class="text-[11px] text-[#666666] font-mono truncate">{node.address.clone()}</p>
                                        </div>
                                    </div>

                                    <div class="flex items-center space-x-2 flex-shrink-0">
                                        {if let Some(ms) = node.ping_ms {
                                            view! {
                                                <span class="text-xs font-mono text-[#1de966]">{format!("{}ms", ms)}</span>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }}

                                        <button
                                            on:click=on_select
                                            class=if is_active {
                                                "px-2.5 py-1 rounded text-xs font-medium bg-[#18823d]/20 text-[#1de966] border border-[#1de966]/40 cursor-default"
                                            } else {
                                                "px-2.5 py-1 rounded text-xs font-medium bg-[#222222] text-[#ababab] border border-[#2c2c2c] hover:text-white hover:border-[#3a3a3a] transition-colors"
                                            }
                                        >
                                            {if is_active { "Connected" } else { "Select" }}
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // 5. Routing Rules Card (GeoIP / GeoSite)
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-4 space-y-3">
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="text-sm font-semibold text-white">"Xray Routing Rules"</h3>
                        <p class="text-xs text-[#ababab]">"Configure routing for domains, subnets and categories (+ = proxy via VPN, - = direct bypass)"</p>
                    </div>
                </div>

                <div class="flex flex-wrap gap-2 pt-1">
                    <span class="px-2 py-1 rounded text-xs font-mono bg-[#222222] text-[#4e97fa] border border-[#2c2c2c]">
                        "+ geosite:blocked"
                    </span>
                    <span class="px-2 py-1 rounded text-xs font-mono bg-[#222222] text-[#4e97fa] border border-[#2c2c2c]">
                        "+ geoip:telegram"
                    </span>
                    <span class="px-2 py-1 rounded text-xs font-mono bg-[#222222] text-[#ababab] border border-[#2c2c2c]">
                        "- geoip:ru"
                    </span>
                    <span class="px-2 py-1 rounded text-xs font-mono bg-[#222222] text-[#ababab] border border-[#2c2c2c]">
                        "- LAN"
                    </span>
                    <span class="px-2 py-1 rounded text-xs font-mono bg-[#222222] text-[#ababab] border border-[#2c2c2c]">
                        "- Localhost"
                    </span>
                </div>
            </div>
        </div>
    }
}
