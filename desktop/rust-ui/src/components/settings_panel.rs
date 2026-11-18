//! Settings view component matching Portmaster's layout and settings list.

use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn SettingsPanel() -> impl IntoView {
    let state = expect_context::<AppState>();
    let route_by_default = state.route_by_default;
    let on_toggle_route_default = {
        let state = state.clone();
        move |_| state.toggle_route_by_default()
    };

    // State for mock settings
    let (filter_action, set_filter_action) = signal("permit".to_string());
    let (block_lan, set_block_lan) = signal(false);
    let (block_p2p, set_block_p2p) = signal(false);
    let (xray_binary_path, set_xray_binary_path) = signal("/usr/bin/xray-rust".to_string());
    let (dns_server, set_dns_server) = signal("tls://1.1.1.1".to_string());

    view! {
        <div class="space-y-6">
            // Settings Category: VPN (Xray)
            <div class="space-y-3">
                <div class="pb-1 border-b border-[#2c2c2c]">
                    <h2 class="text-sm font-semibold text-white">"VPN (Xray) Configuration"</h2>
                    <p class="text-xs text-[#666666]">"Manage gateway execution and default routing policies"</p>
                </div>

                <div class="space-y-2">
                    // Setting 1: Route Through VPN (Xray) by Default
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"Route Through VPN (Xray) by Default"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"spn/use"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "Route network traffic through VPN (Xray) by default. If disabled, traffic connects directly bypassing VPN (unless allowed by Xray rules or per-app settings)."
                            </p>
                        </div>
                        <button
                            type="button"
                            on:click=on_toggle_route_default
                            class=move || {
                                if route_by_default.get() {
                                    "w-10 h-5 rounded-full bg-[#4e97fa] p-0.5 transition-colors cursor-pointer flex items-center"
                                } else {
                                    "w-10 h-5 rounded-full bg-[#2c2c2c] p-0.5 transition-colors cursor-pointer flex items-center"
                                }
                            }
                        >
                            <span class=move || {
                                if route_by_default.get() {
                                    "w-4 h-4 rounded-full bg-white shadow-sm transform translate-x-5 transition-transform"
                                } else {
                                    "w-4 h-4 rounded-full bg-[#ababab] shadow-sm transform translate-x-0 transition-transform"
                                }
                            }></span>
                        </button>
                    </div>

                    // Setting 2: Xray Binary (Single entry! No duplicate!)
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4 flex-wrap">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"Xray Binary"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"xray/binary"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "Path to the Xray binary. Auto-detect checks PATH, /usr/bin/xray-rust, /opt/v2rayn-bin, and local bin directory."
                            </p>
                        </div>
                        <div class="flex items-center space-x-2">
                            <input
                                type="text"
                                prop:value=move || xray_binary_path.get()
                                on:input=move |ev| set_xray_binary_path.set(event_target_value(&ev))
                                class="bg-[#121213] border border-[#2c2c2c] rounded px-3 py-1 text-xs text-white font-mono w-56 focus:border-[#4e97fa] focus:outline-none"
                            />
                        </div>
                    </div>
                </div>
            </div>

            // Settings Category: Firewall & Default Actions
            <div class="space-y-3">
                <div class="pb-1 border-b border-[#2c2c2c]">
                    <h2 class="text-sm font-semibold text-white">"Firewall & Network Protection"</h2>
                    <p class="text-xs text-[#666666]">"Default network rules and filtering behaviors"</p>
                </div>

                <div class="space-y-2">
                    // Setting: Default Network Action
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"Default Network Action"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"filter/defaultAction"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "The default network action applied when nothing else explicitly allows or blocks a connection."
                            </p>
                        </div>
                        <select
                            prop:value=move || filter_action.get()
                            on:change=move |ev| set_filter_action.set(event_target_value(&ev))
                            class="bg-[#121213] border border-[#2c2c2c] text-xs text-white rounded px-2.5 py-1 focus:border-[#4e97fa] focus:outline-none"
                        >
                            <option value="permit">"Permit"</option>
                            <option value="block">"Block"</option>
                            <option value="ask">"Prompt / Ask"</option>
                        </select>
                    </div>

                    // Setting: Force Block LAN
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"Force Block LAN"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"filter/blockLAN"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "Force block all connections to and from the Local Area Network (LAN)."
                            </p>
                        </div>
                        <button
                            type="button"
                            on:click=move |_| set_block_lan.set(!block_lan.get())
                            class=move || {
                                if block_lan.get() {
                                    "w-10 h-5 rounded-full bg-[#4e97fa] p-0.5 transition-colors cursor-pointer flex items-center"
                                } else {
                                    "w-10 h-5 rounded-full bg-[#2c2c2c] p-0.5 transition-colors cursor-pointer flex items-center"
                                }
                            }
                        >
                            <span class=move || {
                                if block_lan.get() {
                                    "w-4 h-4 rounded-full bg-white shadow-sm transform translate-x-5 transition-transform"
                                } else {
                                    "w-4 h-4 rounded-full bg-[#ababab] shadow-sm transform translate-x-0 transition-transform"
                                }
                            }></span>
                        </button>
                    </div>

                    // Setting: Block P2P
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"Block Peer-to-Peer (P2P)"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"filter/blockP2P"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "Block direct incoming and outgoing peer-to-peer communication."
                            </p>
                        </div>
                        <button
                            type="button"
                            on:click=move |_| set_block_p2p.set(!block_p2p.get())
                            class=move || {
                                if block_p2p.get() {
                                    "w-10 h-5 rounded-full bg-[#4e97fa] p-0.5 transition-colors cursor-pointer flex items-center"
                                } else {
                                    "w-10 h-5 rounded-full bg-[#2c2c2c] p-0.5 transition-colors cursor-pointer flex items-center"
                                }
                            }
                        >
                            <span class=move || {
                                if block_p2p.get() {
                                    "w-4 h-4 rounded-full bg-white shadow-sm transform translate-x-5 transition-transform"
                                } else {
                                    "w-4 h-4 rounded-full bg-[#ababab] shadow-sm transform translate-x-0 transition-transform"
                                }
                            }></span>
                        </button>
                    </div>
                </div>
            </div>

            // Settings Category: DNS Resolver
            <div class="space-y-3">
                <div class="pb-1 border-b border-[#2c2c2c]">
                    <h2 class="text-sm font-semibold text-white">"DNS Resolver & Privacy"</h2>
                    <p class="text-xs text-[#666666]">"Upstream DNS servers, DoT and DoH encryption"</p>
                </div>

                <div class="space-y-2">
                    <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg p-3.5 flex items-center justify-between gap-4 flex-wrap">
                        <div class="max-w-2xl">
                            <div class="flex items-center space-x-2">
                                <span class="text-xs font-semibold text-white">"DNS Upstream Server"</span>
                                <span class="text-[10px] text-[#666666] font-mono">"dns/servers"</span>
                            </div>
                            <p class="text-xs text-[#ababab] mt-0.5">
                                "Encrypted DNS servers used for name resolution. Supports tls:// (DNS-over-TLS) and https:// (DNS-over-HTTPS)."
                            </p>
                        </div>
                        <input
                            type="text"
                            prop:value=move || dns_server.get()
                            on:input=move |ev| set_dns_server.set(event_target_value(&ev))
                            class="bg-[#121213] border border-[#2c2c2c] rounded px-3 py-1 text-xs text-white font-mono w-56 focus:border-[#4e97fa] focus:outline-none"
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
