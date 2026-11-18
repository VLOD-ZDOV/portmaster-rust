//! Network Activity / Monitor view component matching Portmaster's live connection monitor.

use leptos::prelude::*;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ConnItem {
    pub id: &'static str,
    pub process: &'static str,
    pub destination: &'static str,
    pub country: &'static str,
    pub protocol: &'static str,
    pub verdict: &'static str, // "allowed", "blocked", "tunneled"
    pub time_ago: &'static str,
}

#[component]
pub fn MonitorPanel() -> impl IntoView {
    let mock_connections = vec![
        ConnItem {
            id: "c1",
            process: "telegram-desktop",
            destination: "149.154.167.92:443 (Telegram Router)",
            country: "NL",
            protocol: "TCP",
            verdict: "tunneled",
            time_ago: "just now",
        },
        ConnItem {
            id: "c2",
            process: "firefox",
            destination: "github.com:443 (140.82.121.4)",
            country: "US",
            protocol: "TLS",
            verdict: "tunneled",
            time_ago: "2s ago",
        },
        ConnItem {
            id: "c3",
            process: "discord",
            destination: "gateway.discord.gg:443",
            country: "US",
            protocol: "WSS",
            verdict: "allowed",
            time_ago: "5s ago",
        },
        ConnItem {
            id: "c4",
            process: "steam",
            destination: "cm01-fra1.cm.steampowered.com:27018",
            country: "DE",
            protocol: "UDP",
            verdict: "allowed",
            time_ago: "12s ago",
        },
        ConnItem {
            id: "c5",
            process: "telemetry-agent",
            destination: "telemetry.tracker.io:443",
            country: "US",
            protocol: "HTTPS",
            verdict: "blocked",
            time_ago: "24s ago",
        },
    ];

    let (connections, _set_connections) = signal(mock_connections);

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between pb-3 border-b border-[#2c2c2c]">
                <div>
                    <h2 class="text-sm font-semibold text-white">"Live Network Connections"</h2>
                    <p class="text-xs text-[#666666]">"Real-time packet inspection and firewall verdicts"</p>
                </div>
                <span class="px-2 py-0.5 rounded text-xs font-mono bg-[#18823d]/20 text-[#1de966] border border-[#1de966]/30">
                    "Stream Live"
                </span>
            </div>

            // Connections Table
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded-lg overflow-hidden">
                <table class="w-full text-left text-xs">
                    <thead class="bg-[#222222] border-b border-[#2c2c2c] text-[#ababab]">
                        <tr>
                            <th class="px-4 py-2.5 font-medium">"Application"</th>
                            <th class="px-4 py-2.5 font-medium">"Destination"</th>
                            <th class="px-4 py-2.5 font-medium">"Protocol"</th>
                            <th class="px-4 py-2.5 font-medium">"Verdict"</th>
                            <th class="px-4 py-2.5 font-medium text-right">"Time"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-[#2c2c2c]">
                        {move || {
                            connections.get().into_iter().map(|item| {
                                view! {
                                    <tr class="hover:bg-[#222222]/50 transition-colors">
                                        <td class="px-4 py-2.5 font-medium text-white flex items-center space-x-2">
                                            <span class="w-2 h-2 rounded-full bg-[#4e97fa]"></span>
                                            <span>{item.process}</span>
                                        </td>
                                        <td class="px-4 py-2.5 text-[#ababab] font-mono">
                                            <span>{item.destination}</span>
                                            <span class="ml-1.5 px-1 rounded bg-[#222222] text-[10px] text-[#666666]">{item.country}</span>
                                        </td>
                                        <td class="px-4 py-2.5 text-xs text-[#ababab] font-mono">{item.protocol}</td>
                                        <td class="px-4 py-2.5">
                                            {match item.verdict {
                                                "tunneled" => view! {
                                                    <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-[#4e97fa]/15 text-[#4e97fa] border border-[#4e97fa]/30">
                                                        "VPN Routed"
                                                    </span>
                                                }.into_any(),
                                                "allowed" => view! {
                                                    <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-[#18823d]/20 text-[#1de966] border border-[#1de966]/30">
                                                        "Direct"
                                                    </span>
                                                }.into_any(),
                                                "blocked" => view! {
                                                    <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-[#e01d1d]/20 text-[#e01d1d] border border-[#e01d1d]/30">
                                                        "Blocked"
                                                    </span>
                                                }.into_any(),
                                                _ => view! { <span/> }.into_any(),
                                            }}
                                        </td>
                                        <td class="px-4 py-2.5 text-right text-[11px] text-[#666666] font-mono">{item.time_ago}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
