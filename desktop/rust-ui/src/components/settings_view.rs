//! Global Settings component matching Portmaster 1:1.
//!
//! Subsystems navigation, search filter, expertise level filter,
//! input controls for all setting types, modified indicator, and "Reset to default".

use leptos::prelude::*;
use pm_models::settings::{ExpertiseLevel, OptionType, Setting, SUBSYSTEMS};

use crate::state::AppState;

#[component]
pub fn SettingsView() -> impl IntoView {
    let state = expect_context::<AppState>();
    let settings = state.settings;
    let expertise_level = state.expertise_level;
    let set_expertise_level = state.set_expertise_level;

    // Active subsystem filter: default to "filter"
    let (active_subsystem, set_active_subsystem) = signal("filter".to_string());
    // Local search term
    let (search_term, set_search_term) = signal(String::new());

    view! {
        <div class="space-y-4">
            // TOP TOOLBAR: Search & Expertise Switcher
            <div class="flex items-center justify-between gap-3 pb-3 border-b border-[#2c2c2c] flex-wrap">
                // Search box
                <div class="relative flex-1 min-w-[240px] max-w-sm">
                    <input
                        type="text"
                        placeholder="Search settings..."
                        prop:value=move || search_term.get()
                        on:input=move |ev| set_search_term.set(event_target_value(&ev))
                        class="input-pm !pl-8 text-xs"
                    />
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        class="absolute w-3.5 h-3.5 top-2.5 left-2.5 text-[#888888]"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
                            clip-rule="evenodd"
                        />
                    </svg>
                </div>

                // Right controls: Expertise Switcher
                <div class="flex items-center space-x-2 text-xs">
                    <span class="text-[#888888]">"Expertise:"</span>
                    <div class="flex items-center rounded bg-[#1b1b1b] p-0.5 border border-[#2c2c2c]">
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::User)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::User {
                                    "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                                } else {
                                    "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "User"
                        </button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Expert)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::Expert {
                                    "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                                } else {
                                    "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "Advanced"
                        </button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Developer)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::Developer {
                                    "px-2.5 py-0.5 rounded-[2px] bg-[#2c2c2c] text-white font-medium text-[11px]"
                                } else {
                                    "px-2.5 py-0.5 rounded-[2px] text-[#ababab] hover:text-white transition-colors text-[11px]"
                                }
                            }
                        >
                            "Developer"
                        </button>
                    </div>
                </div>
            </div>

            // LAYOUT: Subsystems Sidebar + Settings List
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 items-start">
                // LEFT: Subsystems Sidebar
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
                                class=move || {
                                    if active_subsystem.get() == sub_id {
                                        "w-full text-left px-2.5 py-1.5 rounded-[2px] bg-[#2c2c2c] text-white text-xs font-semibold flex items-center justify-between"
                                    } else {
                                        "w-full text-left px-2.5 py-1.5 rounded-[2px] text-[#ababab] hover:bg-[#222222] hover:text-white text-xs transition-colors flex items-center justify-between"
                                    }
                                }
                            >
                                <span>{sub_name}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>

                // RIGHT: Settings List
                <div class="md:col-span-3 space-y-2">
                    {move || {
                        let sub = active_subsystem.get();
                        let local_q = search_term.get().to_lowercase();
                        let global_q = state.filter_query.get().to_lowercase();
                        let query = if !local_q.is_empty() { local_q } else { global_q };
                        let current_exp = expertise_level.get();
                        let all_settings = settings.get();

                        let exp_num = current_exp as u8;
                        let filtered: Vec<_> = all_settings
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
                                    "No settings found for the current subsystem and expertise level."
                                </div>
                            }.into_any();
                        }

                        filtered
                            .into_iter()
                            .map(|s| {
                                view! {
                                    <SettingRow setting=s />
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </div>
            </div>
        </div>
    }
}

/// Single setting row matching Portmaster's exact card & field styling.
#[component]
fn SettingRow(setting: Setting) -> impl IntoView {
    let state = expect_context::<AppState>();
    let s_key = setting.key.clone();
    let is_modified = setting.is_user_configured();

    let state_save = state.clone();
    let state_reset = state.clone();
    let key_save = s_key.clone();
    let key_reset = s_key.clone();

    // Container style: Portmaster shows blue left border if modified
    let container_class = if is_modified {
        "p-3.5 bg-[#1b1b1b] border-l-2 border-[#4e97fa] border-y border-r border-[#2c2c2c] rounded space-y-2.5 transition-all"
    } else {
        "p-3.5 bg-[#1b1b1b] border border-[#2c2c2c] rounded space-y-2.5 transition-all"
    };

    view! {
        <div class=container_class>
            // Header: Name, Unit/Tag, Modified Indicator, Reset Button
            <div class="flex items-start justify-between gap-3">
                <div class="space-y-0.5">
                    <div class="flex items-center space-x-2">
                        <span class="text-xs font-semibold text-white">{setting.name.clone()}</span>
                        {if let Some(unit) = setting.unit() {
                            let u = unit.to_string();
                            view! {
                                <span class="px-1.5 py-0.5 rounded bg-[#222222] text-[#888888] text-[9px] font-mono">
                                    {u}
                                </span>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                    </div>
                    <div class="text-[11px] text-[#ababab] leading-relaxed max-w-xl">
                        {setting.description.clone()}
                    </div>
                </div>

                // Modified badge and Reset to Default button
                <div class="flex items-center space-x-2 flex-shrink-0">
                    {if is_modified {
                        let k = key_reset.clone();
                        view! {
                            <button
                                on:click=move |_| state_reset.reset_setting(&k)
                                title="Reset to system default"
                                class="btn-pm text-[10px] !bg-[#222222] hover:!bg-[#2c2c2c] text-[#4e97fa]"
                            >
                                "Reset to default"
                            </button>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }}
                </div>
            </div>

            // Setting Input Widget based on OptionType
            <div class="pt-1">
                {match setting.opt_type {
                    // 1. Bool Toggle
                    OptionType::Bool => {
                        let cur_val = setting.effective_value().as_bool().unwrap_or(false);
                        let k = key_save.clone();
                        let s = state_save.clone();
                        let next_val = !cur_val;

                        view! {
                            <button
                                on:click=move |_| s.update_setting(&k, serde_json::json!(next_val))
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

                    // 2. String (Dropdown if possible values exist, else input)
                    OptionType::String => {
                        if !setting.possible_values.is_empty() {
                            let cur_val = setting.effective_value().as_str().unwrap_or("").to_string();
                            let k = key_save.clone();
                            let s = state_save.clone();

                            view! {
                                <select
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        s.update_setting(&k, serde_json::json!(val));
                                    }
                                    class="input-pm max-w-sm"
                                >
                                    {setting.possible_values.iter().map(|pv| {
                                        let v_str = pv.value.as_str().unwrap_or("").to_string();
                                        let is_selected = v_str == cur_val;
                                        view! {
                                            <option value=v_str.clone() selected=is_selected>
                                                {pv.name.clone()}
                                            </option>
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
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        s.update_setting(&k, serde_json::json!(val));
                                    }
                                    class="input-pm max-w-md"
                                />
                            }.into_any()
                        }
                    }

                    // 3. Int Number Input
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

                    // 4. String Array (Rule lists / multiple strings)
                    OptionType::StringArray => {
                        let items: Vec<String> = setting
                            .effective_value()
                            .as_array()
                            .map(|arr| {
                                arr.iter().filter_map(|val| val.as_str().map(|s| s.to_string())).collect()
                            })
                            .unwrap_or_default();

                        let (new_item, set_new_item) = signal(String::new());
                        let k = key_save.clone();
                        let s = state_save.clone();
                        let items_clone = items.clone();

                        view! {
                            <div class="space-y-2">
                                <div class="flex flex-wrap gap-1.5">
                                    {items.into_iter().enumerate().map(|(idx, item)| {
                                        let k2 = k.clone();
                                        let s2 = s.clone();
                                        let mut updated = items_clone.clone();

                                        view! {
                                            <span class="inline-flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#222222] border border-[#2c2c2c] text-xs font-mono text-white">
                                                <span>{item}</span>
                                                <button
                                                    on:click=move |_| {
                                                        if idx < updated.len() {
                                                            updated.remove(idx);
                                                            s2.update_setting(&k2, serde_json::json!(updated));
                                                        }
                                                    }
                                                    class="text-[#888888] hover:text-[#e01d1d] font-bold"
                                                >
                                                    "×"
                                                </button>
                                            </span>
                                        }
                                    }).collect_view()}
                                </div>

                                <div class="flex items-center space-x-2 max-w-md">
                                    <input
                                        type="text"
                                        placeholder="Add rule (e.g. 1.1.1.1 or *.domain.com)..."
                                        prop:value=move || new_item.get()
                                        on:input=move |ev| set_new_item.set(event_target_value(&ev))
                                        class="input-pm text-xs"
                                    />
                                    <button
                                        on:click={
                                            let k_add = k.clone();
                                            let s_add = s.clone();
                                            let default_items = items_clone.clone();
                                            move |_| {
                                                let val = new_item.get().trim().to_string();
                                                if !val.is_empty() {
                                                    let mut updated = s_add.settings.get()
                                                        .into_iter()
                                                        .find(|item| item.key == k_add)
                                                        .and_then(|item| item.effective_value().as_array().cloned())
                                                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                                                        .unwrap_or_else(|| default_items.clone());
                                                    updated.push(val);
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
