//! Global Settings component matching Portmaster 1:1.
//! Includes: UI Switcher + Live RAM comparison, Keep API Port :817 toggle,
//! RU/EN switcher, Global Settings Export/Import (#388), DNS append-to-bottom checkbox,
//! and 1-click friendly rule presets (#1280, #2324, Xray GeoSite/GeoIP).

use leptos::prelude::*;
use pm_models::settings::{ExpertiseLevel, OptionType, Setting, SUBSYSTEMS};

use crate::i18n::{t, Lang};
use crate::state::AppState;

const DNS_PRESETS: &[(&str, &[&str])] = &[
    ("Cloudflare DoT", &["dot://1.1.1.1?name=Cloudflare&blockedif=zeroip", "dot://1.0.0.1?name=Cloudflare&blockedif=zeroip"]),
    ("Quad9 DoT", &["dot://9.9.9.9?name=Quad9&blockedif=zeroip", "dot://149.112.112.112?name=Quad9&blockedif=zeroip"]),
    ("AdGuard DoT", &["dot://94.140.14.14?name=AdGuard&blockedif=zeroip", "dot://94.140.15.15?name=AdGuard&blockedif=zeroip"]),
    ("Google DoT", &["dot://8.8.8.8?name=Google", "dot://8.8.4.4?name=Google"]),
];

const XRAY_PRESETS: &[(&str, &[&str])] = &[
    ("RU Direct (-)", &["- geoip:ru", "- geosite:ru"]),
    ("Block Ads (-)", &["- geosite:category-ads-all"]),
    ("YouTube/Discord/TG/AI (+)", &["+ geosite:youtube", "+ geosite:discord", "+ geosite:telegram", "+ geosite:openai"]),
];

const POLICY_PRESETS: &[(&str, &[&str])] = &[
    ("RU + LAN Direct (-)", &["- LAN", "- RU", "- .ru", "- .su", "- .рф"]),
    ("YouTube / Google (+)", &["+ .youtube.com", "+ .googlevideo.com", "+ .ytimg.com"]),
    ("Discord (+)", &["+ .discord.com", "+ .discordapp.com", "+ .discord.gg"]),
    ("Telegram (+)", &["+ .telegram.org", "+ .t.me", "+ AS62041", "+ AS62014"]),
    ("OpenAI / Claude (+)", &["+ .openai.com", "+ .chatgpt.com", "+ .anthropic.com", "+ .claude.ai"]),
];

const ENDPOINT_PRESETS: &[(&str, &[&str])] = &[
    ("Allow LAN & KDE Connect (#2324)", &["+ LAN", "+ 1714-1764/udp"]),
    ("Allow GitHub & crates.io (+)", &["+ .github.com", "+ .githubusercontent.com", "+ .crates.io"]),
    ("Block Telemetry & Ads (-)", &["- .doubleclick.net", "- .google-analytics.com", "- .telemetry.mozilla.org"]),
];

#[component]
pub fn SettingsView() -> impl IntoView {
    let state = expect_context::<AppState>();
    let settings = state.settings;
    let expertise_level = state.expertise_level;
    let set_expertise_level = state.set_expertise_level;
    let ui_report = state.ui_report;
    let lang = state.lang;

    let (active_subsystem, set_active_subsystem) = signal("filter".to_string());
    let (search_term, set_search_term) = signal(String::new());
    let (show_import, set_show_import) = signal(false);
    let (import_raw, set_import_raw) = signal(String::new());

    let s_refresh = state.clone();
    Effect::new(move |_| {
        s_refresh.fetch_ui_report();
    });

    view! {
        <div class="space-y-4">
            // TOP TOOLBAR: Search, Language, Export/Import (#388), Expertise
            <div class="flex items-center justify-between gap-3 pb-3 border-b border-[#2c2c2c] flex-wrap">
                <div class="relative flex-1 min-w-[220px] max-w-sm">
                    <input
                        type="text"
                        prop:placeholder=move || t(lang.get(), "set.search")
                        prop:value=move || search_term.get()
                        on:input=move |ev| set_search_term.set(event_target_value(&ev))
                        class="input-pm !pl-8 text-xs"
                    />
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="absolute w-3.5 h-3.5 top-2.5 left-2.5 text-[#888888]">
                        <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd" />
                    </svg>
                </div>

                <div class="flex flex-wrap items-center gap-2 text-xs">
                    // Language toggle
                    <div class="flex items-center rounded bg-[#1b1b1b] p-0.5 border border-[#2c2c2c]">
                        <button
                            on:click={ let s = state.clone(); move |_| s.set_language(Lang::Ru) }
                            class=move || if lang.get() == Lang::Ru {
                                "px-2 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                            } else {
                                "px-2 py-0.5 rounded-[2px] text-[#ababab] hover:text-white text-[11px]"
                            }
                        >"RU"</button>
                        <button
                            on:click={ let s = state.clone(); move |_| s.set_language(Lang::En) }
                            class=move || if lang.get() == Lang::En {
                                "px-2 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                            } else {
                                "px-2 py-0.5 rounded-[2px] text-[#ababab] hover:text-white text-[11px]"
                            }
                        >"EN"</button>
                    </div>

                    // Export / Import Settings (#388)
                    <button
                        on:click={ let s = state.clone(); move |_| s.export_settings_sync() }
                        class="btn-pm !bg-[#222222] text-[#4e97fa] border border-[#2c2c2c] text-[11px]"
                    >
                        {move || t(lang.get(), "set.export")}
                    </button>
                    <button
                        on:click=move |_| set_show_import.update(|v| *v = !*v)
                        class="btn-pm !bg-[#222222] text-[#ababab] border border-[#2c2c2c] text-[11px] hover:text-white"
                    >
                        {move || t(lang.get(), "set.import")}
                    </button>

                    // Expertise Switcher
                    <div class="flex items-center rounded bg-[#1b1b1b] p-0.5 border border-[#2c2c2c]">
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::User)
                            class=move || if expertise_level.get() == ExpertiseLevel::User {
                                "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                            } else {
                                "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white text-[11px]"
                            }
                        >"User"</button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Expert)
                            class=move || if expertise_level.get() == ExpertiseLevel::Expert {
                                "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                            } else {
                                "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white text-[11px]"
                            }
                        >"Expert"</button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Developer)
                            class=move || if expertise_level.get() == ExpertiseLevel::Developer {
                                "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                            } else {
                                "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white text-[11px]"
                            }
                        >"Dev"</button>
                    </div>
                </div>
            </div>

            // Import Settings inline box
            {
                let s_imp = state.clone();
                move || if show_import.get() {
                    let s = s_imp.clone();
                    view! {
                        <div class="bg-[#1b1b1b] border border-[#4e97fa]/40 rounded p-3 space-y-2">
                            <textarea
                                rows="4"
                                placeholder="Paste exported Portmaster settings (JSON/YAML)..."
                                prop:value=move || import_raw.get()
                                on:input=move |ev| set_import_raw.set(event_target_value(&ev))
                                class="input-pm font-mono text-[11px] w-full"
                            />
                            <div class="flex justify-end gap-2">
                                <button on:click=move |_| set_show_import.set(false) class="btn-pm !bg-[#222222] text-[#888888]">"Cancel"</button>
                                <button
                                    on:click=move |_| {
                                        let raw = import_raw.get();
                                        if !raw.trim().is_empty() {
                                            s.import_settings_sync(raw);
                                            set_import_raw.set(String::new());
                                            set_show_import.set(false);
                                        }
                                    }
                                    class="btn-pm !bg-[#4e97fa] text-white font-semibold"
                                >
                                    {move || t(lang.get(), "set.import")}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }
            }

            // UI SWITCHER + LIVE RAM COMPARISON + SECURITY PORT :817 TOGGLE
            {
                let s_port = state.clone();
                let s_fl = state.clone();
                move || {
                    let Some(report) = ui_report.get() else {
                        return view! { <span/> }.into_any();
                    };
                    let l = lang.get();
                    let keep_open = report.keep_api_port_open;
                    let listening = report.api_port_listening;
                    let port_addr = report.api_port_address.clone();
                    let sp = s_port.clone();
                    let sf = s_fl.clone();

                    view! {
                        <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3.5 space-y-3">
                            <div class="flex flex-wrap items-center justify-between gap-2">
                                <div class="flex flex-wrap items-center gap-2">
                                    <span class="text-xs font-semibold text-white">{t(l, "set.ui_title")}</span>
                                    <span class=if listening {
                                        "px-2 py-0.5 rounded text-[10px] font-mono bg-[#143d24] text-[#1de966] border border-[#1de966]/30"
                                    } else {
                                        "px-2 py-0.5 rounded text-[10px] font-mono bg-[#16345c] text-[#4e97fa] border border-[#4e97fa]/30"
                                    }>
                                        {if listening {
                                            format!("{}: {}", t(l, "set.port_listening"), port_addr)
                                        } else {
                                            t(l, "set.port_closed").to_string()
                                        }}
                                    </span>
                                </div>

                                <label class="inline-flex items-center gap-2 text-xs text-[#cacaca] cursor-pointer">
                                    <input
                                        type="checkbox"
                                        prop:checked=keep_open
                                        on:change=move |_| sp.toggle_keep_api_port(!keep_open)
                                        class="w-4 h-4 accent-[#4e97fa] cursor-pointer"
                                    />
                                    <span>{t(l, "set.port_open")}</span>
                                </label>
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
                                {report.flavors.into_iter().map(|f| {
                                    let sf2 = sf.clone();
                                    let fid = f.id.clone();
                                    let ram_label = if let Some(live) = f.live_ram_mb {
                                        format!("{}: {:.1} MB", t(l, "set.live_ram"), live)
                                    } else {
                                        format!("{}: ~{:.1} MB", t(l, "set.bench_ram"), f.benchmark_ram_mb)
                                    };
                                    view! {
                                        <div class=if f.active {
                                            "p-2.5 rounded bg-[#121213] border border-[#4e97fa] space-y-1.5"
                                        } else {
                                            "p-2.5 rounded bg-[#121213] border border-[#2c2c2c] space-y-1.5"
                                        }>
                                            <div class="flex items-center justify-between gap-1.5">
                                                <span class="text-xs font-semibold text-white truncate">{f.name}</span>
                                                <span class="px-1.5 py-0.5 rounded bg-[#222222] text-[10px] font-mono text-[#4e97fa] whitespace-nowrap">
                                                    {ram_label}
                                                </span>
                                            </div>
                                            <div class="text-[10px] text-[#888888]">{f.kind}</div>
                                            <div class="flex items-center gap-1.5 pt-1">
                                                <button
                                                    disabled=f.active || !f.installed
                                                    on:click=move |_| sf2.switch_ui_flavor(fid.clone())
                                                    class=if f.active {
                                                        "btn-pm !bg-[#143d24] text-[#1de966] text-[10px] cursor-default"
                                                    } else if !f.installed {
                                                        "btn-pm !bg-[#222222] text-[#666666] text-[10px] cursor-not-allowed"
                                                    } else {
                                                        "btn-pm !bg-[#4e97fa] text-white text-[10px]"
                                                    }
                                                >
                                                    {if f.active {
                                                        t(l, "set.active")
                                                    } else if f.installed {
                                                        t(l, "set.activate")
                                                    } else {
                                                        t(l, "set.not_installed")
                                                    }}
                                                </button>
                                                {f.url.filter(|_| f.installed).map(|u| view! {
                                                    <a href=u class="btn-pm !bg-[#222222] text-[#ababab] hover:text-white text-[10px]">
                                                        "Open"
                                                    </a>
                                                })}
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }.into_any()
                }
            }

            // SUBSYSTEMS + SETTINGS LIST
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 items-start">
                <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-2 space-y-1">
                    <div class="px-2 py-1 text-[10px] uppercase font-semibold text-[#888888] tracking-wider">
                        "Subsystems"
                    </div>
                    {SUBSYSTEMS.iter().map(|sub| {
                        let sub_id = sub.id.to_string();
                        let sub_name = sub.name;
                        let s_id = sub_id.clone();
                        view! {
                            <button
                                on:click=move |_| set_active_subsystem.set(s_id.clone())
                                class=move || if active_subsystem.get() == sub_id {
                                    "w-full text-left px-2.5 py-1.5 rounded-[2px] bg-[#2c2c2c] text-white text-xs font-semibold flex items-center justify-between"
                                } else {
                                    "w-full text-left px-2.5 py-1.5 rounded-[2px] text-[#ababab] hover:bg-[#222222] hover:text-white text-xs transition-colors flex items-center justify-between"
                                }
                            >
                                <span>{sub_name}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>

                <div class="md:col-span-3 space-y-2">
                    {move || {
                        let sub = active_subsystem.get();
                        let local_q = search_term.get().to_lowercase();
                        let global_q = state.filter_query.get().to_lowercase();
                        let query = if !local_q.is_empty() { local_q } else { global_q };
                        let exp_num = expertise_level.get() as u8;
                        let filtered: Vec<_> = settings
                            .get()
                            .into_iter()
                            .filter(|s| {
                                if !query.is_empty() {
                                    s.name.to_lowercase().contains(&query)
                                        || s.description.to_lowercase().contains(&query)
                                        || s.key.to_lowercase().contains(&query)
                                } else {
                                    s.subsystem() == sub
                                }
                            })
                            .filter(|s| s.expertise_level <= exp_num)
                            .collect();

                        if filtered.is_empty() {
                            return view! {
                                <div class="p-8 text-center bg-[#1b1b1b] border border-[#2c2c2c] rounded text-[#888888] text-xs">
                                    "No settings found for this subsystem and expertise level."
                                </div>
                            }.into_any();
                        }

                        filtered
                            .into_iter()
                            .map(|s| view! { <SettingRow setting=s /> })
                            .collect_view()
                            .into_any()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn SettingRow(setting: Setting) -> impl IntoView {
    let state = expect_context::<AppState>();
    let s_key = setting.key.clone();
    let is_modified = setting.is_user_configured();
    let lang = state.lang;

    let state_save = state.clone();
    let state_reset = state.clone();
    let key_save = s_key.clone();
    let key_reset = s_key.clone();

    let container_class = if is_modified {
        "p-3.5 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded space-y-2.5 transition-all"
    } else {
        "p-3.5 bg-[#1b1b1b] border border-[#2c2c2c] rounded space-y-2.5 transition-all"
    };

    view! {
        <div class=container_class>
            <div class="flex items-start justify-between gap-3">
                <div class="space-y-0.5">
                    <div class="flex items-center space-x-2">
                        <span class="text-xs font-semibold text-white">{setting.name.clone()}</span>
                        {setting.unit().map(|u| view! {
                            <span class="px-1.5 py-0.5 rounded bg-[#222222] text-[#888888] text-[9px] font-mono">
                                {u.to_string()}
                            </span>
                        })}
                        <span class="text-[10px] text-[#666666] font-mono">{s_key.clone()}</span>
                    </div>
                    <div class="text-[11px] text-[#ababab] leading-relaxed max-w-xl">{setting.description.clone()}</div>
                </div>

                {is_modified.then(|| {
                    let k = key_reset.clone();
                    view! {
                        <button
                            on:click=move |_| state_reset.reset_setting(&k)
                            class="btn-pm text-[10px] !bg-[#222222] hover:!bg-[#2c2c2c] text-[#4e97fa] flex-shrink-0"
                        >
                            "Reset"
                        </button>
                    }
                })}
            </div>

            <div class="pt-1">
                {match setting.opt_type {
                    OptionType::Bool => {
                        let cur_val = setting.effective_value().as_bool().unwrap_or(false);
                        let k = key_save.clone();
                        let s = state_save.clone();
                        view! {
                            <button
                                on:click=move |_| s.update_setting(&k, serde_json::json!(!cur_val))
                                class=if cur_val {
                                    "px-3 py-1 rounded-[2px] bg-[#143d24] text-[#1de966] text-xs font-semibold border border-[#1de966]/40 flex items-center space-x-1.5"
                                } else {
                                    "px-3 py-1 rounded-[2px] bg-[#222222] text-[#888888] hover:text-white text-xs border border-[#2c2c2c] flex items-center space-x-1.5"
                                }
                            >
                                <span class=if cur_val { "w-2 h-2 rounded-full bg-[#1de966]" } else { "w-2 h-2 rounded-full bg-[#888888]" }></span>
                                <span>{if cur_val { "Enabled" } else { "Disabled" }}</span>
                            </button>
                        }.into_any()
                    }
                    OptionType::String => {
                        if !setting.possible_values.is_empty() {
                            let cur_val = setting.effective_value().as_str().unwrap_or("").to_string();
                            let k = key_save.clone();
                            let s = state_save.clone();
                            view! {
                                <select
                                    on:change=move |ev| s.update_setting(&k, serde_json::json!(event_target_value(&ev)))
                                    class="input-pm max-w-sm"
                                >
                                    {setting.possible_values.iter().map(|pv| {
                                        let v_str = pv.value.as_str().unwrap_or("").to_string();
                                        let is_selected = v_str == cur_val;
                                        view! {
                                            <option value=v_str selected=is_selected>{pv.name.clone()}</option>
                                        }
                                    }).collect_view()}
                                </select>
                            }.into_any()
                        } else {
                            let cur_val = setting.effective_value().as_str().unwrap_or("").to_string();
                            let k = key_save.clone();
                            let s = state_save.clone();
                            view! {
                                <input
                                    type="text"
                                    prop:value=cur_val
                                    on:change=move |ev| s.update_setting(&k, serde_json::json!(event_target_value(&ev)))
                                    class="input-pm max-w-md"
                                />
                            }.into_any()
                        }
                    }
                    OptionType::Int => {
                        let cur_val = setting.effective_value().as_i64().unwrap_or(0);
                        let k = key_save.clone();
                        let s = state_save.clone();
                        view! {
                            <input
                                type="number"
                                prop:value=cur_val
                                on:change=move |ev| {
                                    if let Ok(num) = event_target_value(&ev).parse::<i64>() {
                                        s.update_setting(&k, serde_json::json!(num));
                                    }
                                }
                                class="input-pm max-w-xs"
                            />
                        }.into_any()
                    }
                    OptionType::StringArray => {
                        let items: Vec<String> = setting
                            .effective_value()
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();

                        let is_dns = key_save == "dns/nameservers" || key_save == "dns/servers";
                        let is_xray_rules = key_save == "xray/rules";
                        let is_policy = key_save == "spn/usagePolicy";
                        let is_endpoints = key_save == "filter/endpoints";
                        let is_rule_list = is_xray_rules || is_policy || is_endpoints;

                        let (new_item, set_new_item) = signal(String::new());
                        let (rule_prefix, set_rule_prefix) = signal("+".to_string());
                        let (append_mode, set_append_mode) = signal(false);

                        let k = key_save.clone();
                        let s = state_save.clone();
                        let items_clone = items.clone();

                        let presets: Option<&'static [(&'static str, &'static [&'static str])]> = if is_dns {
                            Some(DNS_PRESETS)
                        } else if is_xray_rules {
                            Some(XRAY_PRESETS)
                        } else if is_policy {
                            Some(POLICY_PRESETS)
                        } else if is_endpoints {
                            Some(ENDPOINT_PRESETS)
                        } else {
                            None
                        };

                        view! {
                            <div class="space-y-2.5">
                                // Quick Presets + Append-to-bottom checkbox for DNS
                                {presets.map(|list| {
                                    let k_pre = k.clone();
                                    let s_pre = s.clone();
                                    view! {
                                        <div class="space-y-1.5">
                                            <div class="flex flex-wrap gap-1.5">
                                                {list.iter().map(|(label, vals)| {
                                                    let kp = k_pre.clone();
                                                    let sp = s_pre.clone();
                                                    let vals_vec: Vec<String> = vals.iter().map(|x| (*x).to_string()).collect();
                                                    view! {
                                                        <button
                                                            on:click=move |_| {
                                                                let next = if append_mode.get() || is_rule_list {
                                                                    let mut merged: Vec<String> = sp.settings.get_untracked()
                                                                        .into_iter()
                                                                        .find(|it| it.key == kp)
                                                                        .and_then(|it| it.effective_value().as_array().cloned())
                                                                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                                                        .unwrap_or_default();
                                                                    for v in &vals_vec {
                                                                        if !merged.contains(v) {
                                                                            merged.push(v.clone());
                                                                        }
                                                                    }
                                                                    merged
                                                                } else {
                                                                    vals_vec.clone()
                                                                };
                                                                sp.update_setting(&kp, serde_json::json!(next));
                                                            }
                                                            class="px-2 py-0.5 rounded bg-[#222222] hover:bg-[#2c2c2c] border border-[#2c2c2c] text-[11px] text-[#4e97fa] cursor-pointer"
                                                        >
                                                            {*label}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                            {is_dns.then(|| view! {
                                                <label class="inline-flex items-center gap-1.5 text-[11px] text-[#ababab] cursor-pointer">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || append_mode.get()
                                                        on:change=move |_| set_append_mode.update(|v| *v = !*v)
                                                        class="w-3.5 h-3.5 accent-[#4e97fa]"
                                                    />
                                                    <span>{move || t(lang.get(), "set.append_quick")}</span>
                                                </label>
                                            })}
                                        </div>
                                    }
                                })}

                                // Active items chips
                                <div class="flex flex-wrap gap-1.5">
                                    {items.into_iter().enumerate().map(|(idx, item)| {
                                        let k2 = k.clone();
                                        let s2 = s.clone();
                                        let mut updated = items_clone.clone();
                                        let chip_cls = if item.trim_start().starts_with('+') {
                                            "inline-flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#16345c]/60 border border-[#4e97fa]/40 text-xs font-mono text-white"
                                        } else if item.trim_start().starts_with('-') {
                                            "inline-flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#3d1414]/60 border border-[#e01d1d]/30 text-xs font-mono text-white"
                                        } else {
                                            "inline-flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#222222] border border-[#2c2c2c] text-xs font-mono text-white"
                                        };
                                        view! {
                                            <span class=chip_cls>
                                                <span>{item}</span>
                                                <button
                                                    on:click=move |_| {
                                                        if idx < updated.len() {
                                                            updated.remove(idx);
                                                            s2.update_setting(&k2, serde_json::json!(updated));
                                                        }
                                                    }
                                                    class="text-[#888888] hover:text-[#e01d1d] font-bold cursor-pointer"
                                                >
                                                    "×"
                                                </button>
                                            </span>
                                        }
                                    }).collect_view()}
                                </div>

                                // Friendly rule / entry input
                                <div class="flex flex-wrap items-center gap-2 max-w-lg">
                                    {is_rule_list.then(|| view! {
                                        <select
                                            prop:value=move || rule_prefix.get()
                                            on:change=move |ev| set_rule_prefix.set(event_target_value(&ev))
                                            class="input-pm !w-auto text-xs"
                                        >
                                            <option value="+">{if is_endpoints { "+ Allow" } else { "+ Xray/VPN" }}</option>
                                            <option value="-">{if is_endpoints { "- Block" } else { "- Direct/Bypass" }}</option>
                                        </select>
                                    })}
                                    <input
                                        type="text"
                                        placeholder=if is_dns { "dot://1.1.1.1" } else if is_xray_rules { "geosite:youtube or .example.com" } else { "Pattern (e.g. .example.com, LAN, AS13335)" }
                                        prop:value=move || new_item.get()
                                        on:input=move |ev| set_new_item.set(event_target_value(&ev))
                                        class="input-pm text-xs flex-1 min-w-[200px]"
                                    />
                                    <button
                                        on:click={
                                            let k_add = k.clone();
                                            let s_add = s.clone();
                                            let default_items = items_clone.clone();
                                            move |_| {
                                                let raw = new_item.get().trim().to_string();
                                                if !raw.is_empty() {
                                                    let final_entry = if is_rule_list && !raw.starts_with('+') && !raw.starts_with('-') {
                                                        format!("{} {}", rule_prefix.get(), raw)
                                                    } else {
                                                        raw
                                                    };
                                                    let mut updated = s_add.settings.get()
                                                        .into_iter()
                                                        .find(|item| item.key == k_add)
                                                        .and_then(|item| item.effective_value().as_array().cloned())
                                                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                                                        .unwrap_or_else(|| default_items.clone());
                                                    if !updated.contains(&final_entry) {
                                                        updated.push(final_entry);
                                                    }
                                                    s_add.update_setting(&k_add, serde_json::json!(updated));
                                                    set_new_item.set(String::new());
                                                }
                                            }
                                        }
                                        class="btn-pm text-xs"
                                    >
                                        "Add"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                    _ => view! { <span class="text-xs text-[#888888]">"Raw JSON Value"</span> }.into_any(),
                }}
            </div>
        </div>
    }
}
