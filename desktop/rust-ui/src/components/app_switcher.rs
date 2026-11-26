//! Application list & routing controls matching Portmaster App Overview 1:1.
//! Includes Active/Offline separation, visual block/Xray/Split-Tunnel badges (#2251),
//! and inline Profile Creation & Import (#388).

use leptos::prelude::*;
use pm_models::connection::ConnectionRecord;
use pm_models::profile::AppProfile;

use crate::i18n::t;
use crate::state::AppState;

fn app_has_live(p: &AppProfile, conns: &[ConnectionRecord]) -> bool {
    if p.count_alive_connections() > 0 {
        return true;
    }
    conns.iter().any(|c| {
        c.active
            && (c.profile_id == p.id
                || c.profile_id.ends_with(&format!("/{}", p.id))
                || (!p.name.is_empty() && c.profile_name.eq_ignore_ascii_case(&p.name)))
    })
}

#[component]
pub fn AppSwitcher() -> impl IntoView {
    let state = expect_context::<AppState>();
    let profiles = state.profiles;
    let filter_query = state.filter_query;
    let set_filter_query = state.set_filter_query;
    let route_by_default = state.route_by_default;
    let lang = state.lang;

    // Filter tab: "all", "active", "offline", "vpn", "split", "blocked"
    let (active_tab, set_active_tab) = signal("all".to_string());
    let (show_new_modal, set_show_new_modal) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let (new_path, set_new_path) = signal(String::new());
    let (show_import, set_show_import) = signal(false);
    let (import_raw, set_import_raw) = signal(String::new());

    let global_split = move || {
        state
            .settings
            .get()
            .into_iter()
            .find(|s| s.key == "splittun/use")
            .and_then(|s| s.effective_value().as_bool())
            .unwrap_or(false)
    };

    view! {
        <div class="space-y-4">
            // TOP BAR
            <div class="flex flex-wrap items-center justify-between gap-2 pb-3 border-b border-[#2c2c2c]">
                <div class="flex flex-wrap items-center gap-2">
                    <div class="relative w-56">
                        <input
                            type="text"
                            prop:placeholder=move || t(lang.get(), "apps.search")
                            prop:value=move || filter_query.get()
                            on:input=move |ev| set_filter_query.set(event_target_value(&ev))
                            class="input-pm !pl-8 text-xs"
                        />
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="absolute w-3.5 h-3.5 top-2.5 left-2.5 text-[#888888]">
                            <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd" />
                        </svg>
                    </div>

                    <div class="flex flex-wrap items-center gap-1 bg-[#1b1b1b] p-1 rounded border border-[#2c2c2c] text-xs">
                        {
                            let tabs: [(&'static str, &'static str); 6] = [
                                ("all", "apps.all"),
                                ("active", "apps.active"),
                                ("offline", "apps.offline"),
                                ("vpn", "apps.vpn"),
                                ("split", "apps.split"),
                                ("blocked", "apps.blocked"),
                            ];
                            tabs.into_iter().map(|(id, label_key)| {
                                let id_str = id.to_string();
                                let id_cmp = id_str.clone();
                                view! {
                                    <button
                                        on:click=move |_| set_active_tab.set(id_str.clone())
                                        class=move || if active_tab.get() == id_cmp {
                                            "px-2.5 py-0.5 rounded bg-[#2c2c2c] text-white font-medium text-[11px] cursor-pointer"
                                        } else {
                                            "px-2.5 py-0.5 rounded text-[#ababab] hover:text-white transition-colors text-[11px] cursor-pointer"
                                        }
                                    >
                                        {move || t(lang.get(), label_key)}
                                    </button>
                                }
                            }).collect_view()
                        }
                    </div>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    <button
                        on:click=move |_| {
                            set_show_new_modal.update(|v| *v = !*v);
                            set_show_import.set(false);
                        }
                        class="btn-pm !bg-[#222222] text-[#cacaca] border border-[#2c2c2c] hover:text-white"
                    >
                        {move || t(lang.get(), "apps.new_profile")}
                    </button>
                    <button
                        on:click=move |_| {
                            set_show_import.update(|v| *v = !*v);
                            set_show_new_modal.set(false);
                        }
                        class="btn-pm !bg-[#222222] text-[#4e97fa] border border-[#2c2c2c] hover:text-white"
                    >
                        {move || t(lang.get(), "apps.import_profile")}
                    </button>
                    <button
                        on:click={
                            let s = state.clone();
                            move |_| s.toggle_route_by_default()
                        }
                        class=move || {
                            if route_by_default.get() {
                                "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40 flex items-center space-x-1.5"
                            } else {
                                "btn-pm !bg-[#222222] text-[#888888] hover:text-white flex items-center space-x-1.5 border border-[#2c2c2c]"
                            }
                        }
                    >
                        <span>{move || t(lang.get(), "apps.global_vpn")}</span>
                        <span class="font-bold">{move || if route_by_default.get() { "ON" } else { "OFF" }}</span>
                    </button>
                </div>
            </div>

            // CREATE NEW PROFILE DRAWER
            {
                let s_new = state.clone();
                move || if show_new_modal.get() {
                    let s = s_new.clone();
                    view! {
                        <div class="bg-[#1b1b1b] border border-[#4e97fa]/40 rounded p-3 flex flex-wrap items-center gap-2">
                            <input
                                type="text"
                                placeholder="App Name (e.g. Firefox)"
                                prop:value=move || new_name.get()
                                on:input=move |ev| set_new_name.set(event_target_value(&ev))
                                class="input-pm text-xs w-48"
                            />
                            <input
                                type="text"
                                placeholder="Binary path (e.g. /usr/bin/firefox)"
                                prop:value=move || new_path.get()
                                on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                class="input-pm text-xs flex-1 min-w-[200px]"
                            />
                            <button
                                on:click=move |_| {
                                    s.create_profile(new_name.get(), new_path.get());
                                    set_new_name.set(String::new());
                                    set_new_path.set(String::new());
                                    set_show_new_modal.set(false);
                                }
                                class="btn-pm !bg-[#4e97fa] text-white font-semibold"
                            >
                                "Create"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }
            }

            // IMPORT PROFILE DRAWER (#388)
            {
                let s_imp = state.clone();
                move || if show_import.get() {
                    let s = s_imp.clone();
                    view! {
                        <div class="bg-[#1b1b1b] border border-[#4e97fa]/40 rounded p-3 space-y-2">
                            <textarea
                                rows="4"
                                placeholder="Paste exported profile JSON/YAML..."
                                prop:value=move || import_raw.get()
                                on:input=move |ev| set_import_raw.set(event_target_value(&ev))
                                class="input-pm font-mono text-[11px] w-full"
                            />
                            <div class="flex justify-end gap-2">
                                <button
                                    on:click=move |_| set_show_import.set(false)
                                    class="btn-pm !bg-[#222222] text-[#888888]"
                                >
                                    "Cancel"
                                </button>
                                <button
                                    on:click=move |_| {
                                        let raw = import_raw.get();
                                        if !raw.trim().is_empty() {
                                            s.import_profile_sync(raw);
                                            set_import_raw.set(String::new());
                                            set_show_import.set(false);
                                        }
                                    }
                                    class="btn-pm !bg-[#4e97fa] text-white font-semibold"
                                >
                                    {move || t(lang.get(), "apps.import_profile")}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }
            }

            // APPLICATIONS LIST (Grouped Active vs Offline in "All" tab #2251)
            {move || {
                let l = lang.get();
                let query = filter_query.get().to_lowercase();
                let tab = active_tab.get();
                let list = profiles.get();
                let conns = state.connections.get();
                let g_vpn = route_by_default.get();
                let g_split = global_split();

                let mut filtered: Vec<AppProfile> = list
                    .into_iter()
                    .filter(|p| {
                        if query.is_empty() {
                            true
                        } else {
                            p.name.to_lowercase().contains(&query)
                                || p.id.to_lowercase().contains(&query)
                                || p.presentation_path.to_lowercase().contains(&query)
                        }
                    })
                    .filter(|p| {
                        let live = app_has_live(p, &conns);
                        match tab.as_str() {
                            "active" => live,
                            "offline" => !live,
                            "vpn" => p.route_via_vpn_with_default(g_vpn),
                            "split" => p.route_via_split_tun_with_default(g_split),
                            "blocked" => {
                                p.default_action() == "block"
                                    || p.stats.map(|s| s.blocked_count > 0).unwrap_or(false)
                                    || conns.iter().any(|c| c.is_blocked() && (c.profile_id == p.id || c.profile_id.ends_with(&format!("/{}", p.id))))
                            }
                            _ => true,
                        }
                    })
                    .collect();

                filtered.sort_by(|a, b| {
                    a.display_name()
                        .to_lowercase()
                        .cmp(&b.display_name().to_lowercase())
                });

                if filtered.is_empty() {
                    return view! {
                        <div class="p-8 text-center bg-[#1b1b1b] border border-[#2c2c2c] rounded text-[#888888] text-xs">
                            {t(l, "apps.empty")}
                        </div>
                    }.into_any();
                }

                let render_card = |p: AppProfile| {
                    let app_id = p.id.clone();
                    let app_name = p.display_name().to_string();
                    let initials = p.name.chars().take(2).collect::<String>().to_uppercase();
                    let path_desc = if !p.presentation_path.is_empty() { p.presentation_path.clone() } else { p.source.clone() };
                    let vpn_override = p.vpn_route_override();
                    let is_vpn = p.route_via_vpn_with_default(g_vpn);
                    let is_st = p.route_via_split_tun_with_default(g_split);

                    let (vpn_label, vpn_class) = match vpn_override {
                        None => (
                            if is_vpn { t(l, "apps.vpn_auto_on") } else { t(l, "apps.vpn_auto_off") },
                            if is_vpn {
                                "btn-pm !bg-[#4e97fa]/10 text-[#8ab4f8] border border-[#4e97fa]/25"
                            } else {
                                "btn-pm !bg-[#222222] text-[#888888] hover:text-white"
                            },
                        ),
                        Some(true) => (
                            t(l, "apps.vpn_always"),
                            "btn-pm !bg-[#4e97fa]/20 text-[#4e97fa] border border-[#4e97fa]/40 font-semibold",
                        ),
                        Some(false) => (
                            t(l, "apps.vpn_direct"),
                            "btn-pm !bg-[#342a18] text-[#e9b84a] border border-[#e9b84a]/30",
                        ),
                    };

                    let default_action = p.default_action();
                    let is_blocked = default_action == "block";
                    let is_ask = default_action == "ask";

                    let (live_recv, live_sent, live_active) = {
                        let mut r = 0u64;
                        let mut s = 0u64;
                        let mut a = 0usize;
                        for c in &conns {
                            if c.profile_id == p.id
                                || c.profile_id.ends_with(&format!("/{}", p.id))
                                || c.profile_name.eq_ignore_ascii_case(&p.name)
                            {
                                r = r.saturating_add(c.bytes_received);
                                s = s.saturating_add(c.bytes_sent);
                                if c.active {
                                    a += 1;
                                }
                            }
                        }
                        (r, s, a)
                    };
                    let conn_count = p.count_alive_connections().max(live_active as u64);
                    let (bytes_recv, bytes_sent) = if let Some(st) = p.stats {
                        (st.bytes_received.max(live_recv), st.bytes_sent.max(live_sent))
                    } else {
                        (live_recv, live_sent)
                    };
                    let has_config_changes = p.config.is_some();

                    let state_vpn = state.clone();
                    let app_id_vpn = app_id.clone();
                    let state_action = state.clone();
                    let app_id_act = app_id.clone();
                    let next_action = if is_blocked { "permit".to_string() } else { "block".to_string() };
                    let state_nav = state.clone();
                    let app_id_nav = app_id.clone();

                    view! {
                        <div
                            on:click=move |_| state_nav.select_app(Some(app_id_nav.clone()))
                            class="relative card-header-pm bg-[#202020e0] hover:bg-[#292929b0] rounded-[4px] px-3 py-2.5 flex flex-wrap items-center justify-between gap-2 cursor-pointer border border-transparent hover:border-[#2c2c2c] transition-all group"
                        >
                            <div class="flex items-center space-x-3 min-w-0">
                                <div class=if is_blocked {
                                    "w-8 h-8 rounded bg-[#3d1414] border border-[#e01d1d]/40 flex items-center justify-center text-[#e01d1d] font-semibold text-xs flex-shrink-0"
                                } else {
                                    "w-8 h-8 rounded bg-[#2c2c2c] flex items-center justify-center text-white font-semibold text-xs flex-shrink-0 group-hover:bg-[#343434]"
                                }>
                                    {initials}
                                </div>
                                <div class="min-w-0">
                                    <div class="flex items-center gap-1.5 flex-wrap">
                                        <span class="text-xs font-semibold text-[#cacaca] group-hover:text-white truncate">{app_name}</span>
                                        {is_blocked.then(|| view! {
                                            <span class="px-1.5 py-0.2 rounded bg-[#3d1414] text-[#e01d1d] text-[9px] font-semibold border border-[#e01d1d]/30">
                                                {t(l, "apps.badge_blocked")}
                                            </span>
                                        })}
                                        {is_vpn.then(|| view! {
                                            <span class="px-1.5 py-0.2 rounded bg-[#4e97fa]/15 text-[#4e97fa] text-[9px] font-mono border border-[#4e97fa]/30">
                                                {t(l, "apps.badge_xray")}
                                            </span>
                                        })}
                                        {is_st.then(|| view! {
                                            <span class="px-1.5 py-0.2 rounded bg-[#143d24] text-[#1de966] text-[9px] font-mono border border-[#1de966]/30">
                                                {t(l, "apps.badge_st")}
                                            </span>
                                        })}
                                    </div>
                                    <div class="text-[10px] text-[#888888] truncate font-mono">{path_desc}</div>
                                </div>
                            </div>

                            <div class="flex items-center space-x-2 flex-shrink-0">
                                {(conn_count > 0).then(|| view! {
                                    <span class="px-2 py-0.5 rounded bg-[#2c2c2c] text-[#1de966] text-[10px] font-mono font-medium">
                                        {format!("{} conn", conn_count)}
                                    </span>
                                })}
                                {(bytes_recv > 0 || bytes_sent > 0).then(|| view! {
                                    <span class="px-2 py-0.5 rounded bg-[#222222] border border-[#2c2c2c] text-[10px] font-mono flex items-center space-x-1.5">
                                        <span class="text-[#1de966]">{format!("↓ {}", ConnectionRecord::format_bytes(bytes_recv))}</span>
                                        <span class="text-[#4e97fa]">{format!("↑ {}", ConnectionRecord::format_bytes(bytes_sent))}</span>
                                    </span>
                                })}
                                <button
                                    on:click=move |ev: web_sys::MouseEvent| {
                                        ev.stop_propagation();
                                        state_vpn.toggle_app_vpn(app_id_vpn.clone());
                                    }
                                    class=vpn_class
                                >
                                    {vpn_label}
                                </button>
                                <button
                                    on:click=move |ev: web_sys::MouseEvent| {
                                        ev.stop_propagation();
                                        state_action.set_app_action(app_id_act.clone(), next_action.clone());
                                    }
                                    class=if is_blocked {
                                        "btn-pm !bg-[#3d1414] text-[#e01d1d] border border-[#e01d1d]/40 font-semibold"
                                    } else if is_ask {
                                        "btn-pm !bg-[#3d3a14] text-[#e9d31d] border border-[#e9d31d]/40"
                                    } else {
                                        "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40"
                                    }
                                >
                                    {if is_blocked { t(l, "apps.blocked_btn") } else if is_ask { t(l, "apps.ask_btn") } else { t(l, "apps.permitted") }}
                                </button>
                                {has_config_changes.then(|| view! {
                                    <span title=t(l, "apps.badge_custom") class="w-1.5 h-1.5 rounded-full bg-[#4e97fa]"></span>
                                })}
                            </div>
                        </div>
                    }
                };

                if tab == "all" {
                    let mut active_list = Vec::new();
                    let mut offline_list = Vec::new();
                    for p in filtered {
                        if app_has_live(&p, &conns) {
                            active_list.push(p);
                        } else {
                            offline_list.push(p);
                        }
                    }
                    view! {
                        <div class="space-y-4">
                            {(!active_list.is_empty()).then(|| view! {
                                <div class="space-y-1.5">
                                    <div class="text-[11px] font-semibold uppercase tracking-wider text-[#1de966] px-1">
                                        {format!("{} ({})", t(l, "apps.active_group"), active_list.len())}
                                    </div>
                                    <div class="space-y-1.5">
                                        {active_list.into_iter().map(&render_card).collect_view()}
                                    </div>
                                </div>
                            })}
                            {(!offline_list.is_empty()).then(|| view! {
                                <div class="space-y-1.5">
                                    <div class="text-[11px] font-semibold uppercase tracking-wider text-[#888888] px-1">
                                        {format!("{} ({})", t(l, "apps.offline_group"), offline_list.len())}
                                    </div>
                                    <div class="space-y-1.5">
                                        {offline_list.into_iter().map(&render_card).collect_view()}
                                    </div>
                                </div>
                            })}
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-1.5">
                            {filtered.into_iter().map(&render_card).collect_view()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
