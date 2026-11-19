//! Network Activity & Netquery Monitor Panel matching Portmaster 1:1.
//!
//! Features:
//! - Scope filters (All, Internet, LAN, Local, Inbound)
//! - Verdict filters (All, Permitted, Blocked)
//! - Live search by domain, IP, or application
//! - Rich connections table with verdict status dots, country flags, and data counters
//! - Expandable row detail with blocking reasons, AS owners, and instant rule triggers

use leptos::prelude::*;

use crate::state::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeFilter {
    All,
    Internet,
    Lan,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VerdictFilter {
    All,
    Permitted,
    Blocked,
}

#[component]
pub fn MonitorPanel() -> impl IntoView {
    let state = expect_context::<AppState>();
    let connections = state.connections;

    let (search_query, set_search_query) = signal(String::new());
    let (selected_scope, set_selected_scope) = signal(ScopeFilter::All);
    let (selected_verdict, set_selected_verdict) = signal(VerdictFilter::All);
    let (expanded_conn, set_expanded_conn) = signal(Option::<String>::None);

    let state_clear = state.clone();
    let on_clear_history = move |_| {
        state_clear.clear_history();
    };

    view! {
        <div class="space-y-4">
            // TOP HEADER: Title, Breadcrumbs, History stats & Clear
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-2 border-b border-[#2c2c2c]">
                <div>
                    <div class="text-[11px] text-[#888888] flex items-center space-x-1.5 mb-0.5">
                        <span class="hover:text-white cursor-pointer transition-colors">"Monitor"</span>
                        <span>"/"</span>
                        <span class="text-white font-medium">"Network Activity"</span>
                    </div>
                    <h1 class="text-xl font-semibold text-white tracking-tight">"Network Activity"</h1>
                </div>

                <div class="flex items-center space-x-3 text-xs">
                    <span class="text-[#888888]">
                        {move || {
                            let count = connections.get().len();
                            format!("History: {} connections recorded", count)
                        }}
                    </span>
                    <button
                        on:click=on_clear_history
                        class="btn-pm text-xs !bg-[#222222] hover:!bg-[#2c2c2c] text-[#ababab] hover:text-white"
                    >
                        "Clear History"
                    </button>
                </div>
            </div>

            // FILTERS TOOLBAR
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3 space-y-3">
                <div class="flex flex-col md:flex-row items-center gap-3">
                    // Search bar
                    <div class="relative flex-1 w-full">
                        <input
                            type="text"
                            placeholder="Filter by domain, remote IP, or application..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            class="input-pm !pl-8"
                        />
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                            class="absolute w-4 h-4 top-2.5 left-2.5 text-[#888888]"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </div>

                    // Scope Filters
                    <div class="flex items-center space-x-1 w-full md:w-auto">
                        <span class="text-[11px] text-[#888888] mr-1 hidden sm:inline">"Scope:"</span>
                        <button
                            on:click=move |_| set_selected_scope.set(ScopeFilter::All)
                            class=move || {
                                if selected_scope.get() == ScopeFilter::All {
                                    "btn-pm !bg-[#4e97fa] text-white"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                }
                            }
                        >
                            "All"
                        </button>
                        <button
                            on:click=move |_| set_selected_scope.set(ScopeFilter::Internet)
                            class=move || {
                                if selected_scope.get() == ScopeFilter::Internet {
                                    "btn-pm !bg-[#4e97fa] text-white"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                }
                            }
                        >
                            "Internet"
                        </button>
                        <button
                            on:click=move |_| set_selected_scope.set(ScopeFilter::Lan)
                            class=move || {
                                if selected_scope.get() == ScopeFilter::Lan {
                                    "btn-pm !bg-[#4e97fa] text-white"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                }
                            }
                        >
                            "LAN"
                        </button>
                        <button
                            on:click=move |_| set_selected_scope.set(ScopeFilter::Local)
                            class=move || {
                                if selected_scope.get() == ScopeFilter::Local {
                                    "btn-pm !bg-[#4e97fa] text-white"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                }
                            }
                        >
                            "Local"
                        </button>
                    </div>

                    // Verdict Filters
                    <div class="flex items-center space-x-1 w-full md:w-auto">
                        <span class="text-[11px] text-[#888888] mr-1 hidden sm:inline">"Verdict:"</span>
                        <button
                            on:click=move |_| set_selected_verdict.set(VerdictFilter::All)
                            class=move || {
                                if selected_verdict.get() == VerdictFilter::All {
                                    "btn-pm !bg-[#343434] text-white"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                                }
                            }
                        >
                            "All"
                        </button>
                        <button
                            on:click=move |_| set_selected_verdict.set(VerdictFilter::Permitted)
                            class=move || {
                                if selected_verdict.get() == VerdictFilter::Permitted {
                                    "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-[#1de966]"
                                }
                            }
                        >
                            "● Permitted"
                        </button>
                        <button
                            on:click=move |_| set_selected_verdict.set(VerdictFilter::Blocked)
                            class=move || {
                                if selected_verdict.get() == VerdictFilter::Blocked {
                                    "btn-pm !bg-[#3d1414] text-[#e01d1d] border border-[#e01d1d]/40"
                                } else {
                                    "btn-pm !bg-[#222222] text-[#888888] hover:text-[#e01d1d]"
                                }
                            }
                        >
                            "● Blocked"
                        </button>
                    </div>
                </div>
            </div>

            // CONNECTIONS TABLE
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-xs border-collapse">
                        <thead>
                            <tr class="bg-[#121213] border-b border-[#2c2c2c] text-[#888888] font-medium text-[11px]">
                                <th class="py-2.5 px-3 w-10 text-center">"Status"</th>
                                <th class="py-2.5 px-3">"Target Domain / IP"</th>
                                <th class="py-2.5 px-3">"Location"</th>
                                <th class="py-2.5 px-3">"Application"</th>
                                <th class="py-2.5 px-3">"Protocol"</th>
                                <th class="py-2.5 px-3 text-right">"Transferred"</th>
                                <th class="py-2.5 px-3 text-right">"Time"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-[#2c2c2c]/60">
                            {move || {
                                let query = search_query.get().to_lowercase();
                                let scope = selected_scope.get();
                                let verdict = selected_verdict.get();
                                let conns = connections.get();

                                let filtered: Vec<_> = conns
                                    .into_iter()
                                    .filter(|c| {
                                        // Scope filter
                                        match scope {
                                            ScopeFilter::All => true,
                                            ScopeFilter::Internet => c.scope.eq_ignore_ascii_case("Internet"),
                                            ScopeFilter::Lan => c.scope.eq_ignore_ascii_case("LAN"),
                                            ScopeFilter::Local => c.scope.eq_ignore_ascii_case("Local") || c.scope.eq_ignore_ascii_case("HostLocal"),
                                        }
                                    })
                                    .filter(|c| {
                                        // Verdict filter
                                        match verdict {
                                            VerdictFilter::All => true,
                                            VerdictFilter::Permitted => c.is_permitted(),
                                            VerdictFilter::Blocked => c.is_blocked(),
                                        }
                                    })
                                    .filter(|c| {
                                        // Text search
                                        if query.is_empty() {
                                            true
                                        } else {
                                            c.domain.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
                                                || c.remote_ip.contains(&query)
                                                || c.profile_name.to_lowercase().contains(&query)
                                                || c.profile_id.to_lowercase().contains(&query)
                                        }
                                    })
                                    .collect();

                                if filtered.is_empty() {
                                    return view! {
                                        <tr>
                                            <td colspan="7" class="py-8 text-center text-[#888888] text-xs">
                                                "No connection records matching the current filter."
                                            </td>
                                        </tr>
                                    }.into_any();
                                }

                                let state = state.clone();
                                filtered
                                    .into_iter()
                                    .map(|c| {
                                        let conn_id = c.id.clone();
                                        let is_expanded = move || expanded_conn.get().as_deref() == Some(&conn_id);
                                        let toggle_expand = {
                                            let conn_id = c.id.clone();
                                            move |_| {
                                                if expanded_conn.get().as_deref() == Some(&conn_id) {
                                                    set_expanded_conn.set(None);
                                                } else {
                                                    set_expanded_conn.set(Some(conn_id.clone()));
                                                }
                                            }
                                        };

                                        let verdict_dot = if c.is_permitted() {
                                            "w-2.5 h-2.5 rounded-full bg-[#1de966] inline-block shadow-sm"
                                        } else {
                                            "w-2.5 h-2.5 rounded-full bg-[#e01d1d] inline-block shadow-sm"
                                        };

                                        let domain_display = c.domain.clone().unwrap_or_else(|| c.remote_ip.clone());
                                        let remote_dest = format!("{}:{}", c.remote_ip, c.remote_port);
                                        let country_display = c.country.clone().unwrap_or_else(|| "Local".to_string());
                                        let total_bytes = c.bytes_sent.saturating_add(c.bytes_received);
                                        let bytes_str = if total_bytes == 0 {
                                            "0 B".to_string()
                                        } else if total_bytes < 1024 {
                                            format!("{} B", total_bytes)
                                        } else if total_bytes < 1024 * 1024 {
                                            format!("{:.1} KB", total_bytes as f64 / 1024.0)
                                        } else {
                                            format!("{:.1} MB", total_bytes as f64 / (1024.0 * 1024.0))
                                        };

                                        let state_act = state.clone();
                                        let c_id_allow = c.id.clone();
                                        let c_id_block = c.id.clone();

                                        view! {
                                            <tr
                                                on:click=toggle_expand
                                                class="hover:bg-[#222222] cursor-pointer transition-colors"
                                            >
                                                // Status Dot
                                                <td class="py-2.5 px-3 text-center">
                                                    <span class=verdict_dot></span>
                                                </td>

                                                // Target Domain & Remote IP
                                                <td class="py-2.5 px-3 font-medium text-white">
                                                    <div class="truncate max-w-xs">{domain_display}</div>
                                                    <div class="text-[10px] text-[#888888] font-mono">{remote_dest}</div>
                                                </td>

                                                // Location / Country
                                                <td class="py-2.5 px-3 text-[#ababab]">
                                                    <span class="px-1.5 py-0.5 rounded bg-[#222222] font-mono text-[10px] text-white mr-1.5">
                                                        {country_display}
                                                    </span>
                                                    <span class="text-[11px] text-[#888888]">
                                                        {c.country_name.clone().unwrap_or_else(|| c.scope.clone())}
                                                    </span>
                                                </td>

                                                // Application
                                                <td class="py-2.5 px-3 text-white font-medium">
                                                    <div class="flex items-center space-x-1.5">
                                                        <span class="w-4 h-4 rounded bg-[#2c2c2c] flex items-center justify-center text-[9px] text-[#ababab]">
                                                            {c.profile_name.chars().next().unwrap_or('A')}
                                                        </span>
                                                        <span class="truncate max-w-[120px]">{c.profile_name.clone()}</span>
                                                    </div>
                                                </td>

                                                // Protocol & Port
                                                <td class="py-2.5 px-3 font-mono text-[11px] text-[#ababab]">
                                                    <span class="uppercase">{c.protocol.clone()}</span>
                                                    <span class="text-[#888888]">{format!(" / {}", c.remote_port)}</span>
                                                </td>

                                                // Transferred Bytes
                                                <td class="py-2.5 px-3 text-right font-mono text-[11px] text-[#ababab]">
                                                    {bytes_str}
                                                </td>

                                                // Time
                                                <td class="py-2.5 px-3 text-right text-[11px] text-[#888888] whitespace-nowrap">
                                                    {c.time_ago.clone()}
                                                </td>
                                            </tr>

                                            // Expandable Detail Row
                                            {move || {
                                                if is_expanded() {
                                                    let s1 = state_act.clone();
                                                    let s2 = state_act.clone();
                                                    let id_a = c_id_allow.clone();
                                                    let id_b = c_id_block.clone();

                                                    view! {
                                                        <tr class="bg-[#121213] border-y border-[#2c2c2c]">
                                                            <td colspan="7" class="p-4 space-y-3">
                                                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
                                                                    <div class="space-y-1">
                                                                        <div class="text-[11px] text-[#888888]">"Filter Verdict Reason"</div>
                                                                        <div class="text-white font-medium">{c.verdict_reason.clone()}</div>
                                                                    </div>
                                                                    <div class="space-y-1">
                                                                        <div class="text-[11px] text-[#888888]">"Autonomous System (ASN) / Owner"</div>
                                                                        <div class="text-[#ababab]">{c.as_owner.clone().unwrap_or_else(|| "Unknown / Unassigned".to_string())}</div>
                                                                    </div>
                                                                    <div class="space-y-1">
                                                                        <div class="text-[11px] text-[#888888]">"Direction & Scope"</div>
                                                                        <div class="text-[#ababab]">{format!("{} • {}", c.direction, c.scope)}</div>
                                                                    </div>
                                                                </div>

                                                                <div class="flex items-center space-x-2 pt-2 border-t border-[#2c2c2c]">
                                                                    <button
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            s1.allow_connection(&id_a);
                                                                        }
                                                                        class="btn-pm !bg-[#143d24] text-[#1de966] hover:!bg-[#18823d] border border-[#1de966]/30"
                                                                    >
                                                                        "Always Permit this Domain"
                                                                    </button>
                                                                    <button
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            s2.block_connection(&id_b);
                                                                        }
                                                                        class="btn-pm !bg-[#3d1414] text-[#e01d1d] hover:!bg-[#811818] border border-[#e01d1d]/30"
                                                                    >
                                                                        "Block this Domain"
                                                                    </button>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    view! { <span/> }.into_any()
                                                }
                                            }}
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }}
                        </tbody>
                    </table>
                </div>

                // Bottom Summary Bar
                <div class="p-2.5 bg-[#121213] border-t border-[#2c2c2c] flex items-center justify-between text-xs text-[#888888]">
                    <span>
                        {move || {
                            let count = connections.get().len();
                            format!("Showing {} total connections", count)
                        }}
                    </span>
                    <div class="flex items-center space-x-3 text-[11px]">
                        <span class="flex items-center space-x-1">
                            <span class="w-2 h-2 rounded-full bg-[#1de966]"></span>
                            <span>"Permitted"</span>
                        </span>
                        <span class="flex items-center space-x-1">
                            <span class="w-2 h-2 rounded-full bg-[#e01d1d]"></span>
                            <span>"Blocked"</span>
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
