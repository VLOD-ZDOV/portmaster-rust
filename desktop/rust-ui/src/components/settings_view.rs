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
        <div class="space-y-6">
            // Header Bar: Search, Help link, Expertise Level Switcher
            <div class="flex items-center justify-between gap-4 flex-wrap bg-slate-900/60 p-4 rounded-2xl border border-slate-800">
                // Search box
                <div class="relative flex-1 min-w-[240px]">
                    <input
                        type="text"
                        placeholder="Search settings..."
                        prop:value=move || search_term.get()
                        on:input=move |ev| set_search_term.set(event_target_value(&ev))
                        class="w-full pl-9 pr-3 py-2 rounded-xl bg-slate-950/80 border border-slate-700/80 text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500 transition-all"
                    />
                    <svg class="w-4 h-4 text-slate-500 absolute left-3 top-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                </div>

                // Right controls: Expertise Switcher
                <div class="flex items-center space-x-3 text-xs">
                    <span class="text-slate-400 font-medium">"Expertise:"</span>
                    <div class="flex items-center rounded-xl bg-slate-950 p-1 border border-slate-800">
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::User)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::User {
                                    "px-2.5 py-1 rounded-lg bg-slate-800 text-white font-medium shadow-sm"
                                } else {
                                    "px-2.5 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "User"
                        </button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Expert)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::Expert {
                                    "px-2.5 py-1 rounded-lg bg-cyan-600/30 text-cyan-300 font-medium border border-cyan-500/30 shadow-sm"
                                } else {
                                    "px-2.5 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "Expert"
                        </button>
                        <button
                            on:click=move |_| set_expertise_level.set(ExpertiseLevel::Developer)
                            class=move || {
                                if expertise_level.get() == ExpertiseLevel::Developer {
                                    "px-2.5 py-1 rounded-lg bg-amber-500/20 text-amber-300 font-medium border border-amber-500/30 shadow-sm"
                                } else {
                                    "px-2.5 py-1 rounded-lg text-slate-400 hover:text-slate-200 transition-colors"
                                }
                            }
                        >
                            "Developer"
                        </button>
                    </div>
                </div>
            </div>

            // Main Settings Layout: Subsystem Sidebar + Settings Cards
            <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
                // Left Column: Subsystems Navigation
                <div class="lg:col-span-1 space-y-1.5">
                    <span class="text-[10px] uppercase font-bold text-slate-400 tracking-wider px-2 block mb-2">"Subsystems"</span>
                    {SUBSYSTEMS.iter().map(|sub| {
                        let sub_id = sub.id;
                        let sub_name = sub.name;
                        let is_active = move || active_subsystem.get() == sub_id;

                        view! {
                            <button
                                on:click=move |_| set_active_subsystem.set(sub_id.to_string())
                                class=move || {
                                    if is_active() {
                                        "w-full text-left px-3.5 py-2.5 rounded-xl bg-cyan-500/10 text-cyan-300 border border-cyan-500/30 font-semibold text-xs shadow-sm flex items-center justify-between"
                                    } else {
                                        "w-full text-left px-3.5 py-2.5 rounded-xl bg-slate-900/60 hover:bg-slate-900 text-slate-400 hover:text-slate-200 border border-slate-800 text-xs font-medium transition-colors flex items-center justify-between"
                                    }
                                }
                            >
                                <span>{sub_name}</span>
                                <svg class="w-3.5 h-3.5 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                                </svg>
                            </button>
                        }
                    }).collect_view()}
                </div>

                // Right Column: Filtered Settings
                <div class="lg:col-span-3 space-y-4">
                    {move || {
                        let query = search_term.get().to_lowercase();
                        let current_sub = active_subsystem.get();
                        let max_level = match expertise_level.get() {
                            ExpertiseLevel::User => 0,
                            ExpertiseLevel::Expert => 1,
                            ExpertiseLevel::Developer => 2,
                        };

                        let all = settings.get();
                        let filtered: Vec<_> = all.into_iter().filter(|s| {
                            // Subsystem match if no search query
                            if query.is_empty() && s.subsystem() != current_sub {
                                return false;
                            }

                            // Expertise level filter
                            if s.expertise_level > max_level {
                                return false;
                            }

                            // Search filter
                            if !query.is_empty() {
                                let match_name = s.name.to_lowercase().contains(&query);
                                let match_desc = s.description.to_lowercase().contains(&query);
                                let match_key = s.key.to_lowercase().contains(&query);
                                if !match_name && !match_desc && !match_key {
                                    return false;
                                }
                            }

                            true
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="p-12 text-center rounded-2xl bg-slate-900/40 border border-slate-800 text-slate-500 text-sm">
                                    "No settings found matching your criteria."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    {filtered.into_iter().map(|s| {
                                        view! {
                                            <SettingItem setting=s />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// One rendered Setting card supporting all Portmaster setting types.
#[component]
fn SettingItem(setting: Setting) -> impl IntoView {
    let state = expect_context::<AppState>();
    let key = setting.key.clone();
    let is_modified = setting.is_modified();
    let opt_type = setting.opt_type;
    let name = setting.name.clone();
    let description = setting.description.clone();
    let requires_restart = setting.requires_restart;
    let unit = setting.annotations.unit.clone();
    let possible_values = setting.possible_values.clone();

    // Local state for string-array rule editor
    let (new_rule_input, set_new_rule_input) = signal(String::new());

    let state_reset = state.clone();
    let key_for_reset = key.clone();
    let on_reset = move |_| {
        state_reset.reset_setting(key_for_reset.clone());
    };

    view! {
        <div class=if is_modified {
            "p-5 rounded-2xl bg-slate-900/80 border-l-4 border-l-cyan-500 border border-slate-800 shadow-md transition-all space-y-3"
        } else {
            "p-5 rounded-2xl bg-slate-900/50 border border-slate-800/80 transition-all space-y-3"
        }>
            // Title & Badges
            <div class="flex items-start justify-between gap-3 flex-wrap">
                <div>
                    <div class="flex items-center space-x-2">
                        <h4 class="text-sm font-semibold text-white tracking-tight">{name}</h4>
                        <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-slate-800 text-slate-400">
                            {key.clone()}
                        </span>
                        {if requires_restart {
                            view! {
                                <span class="px-1.5 py-0.5 rounded text-[10px] font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20">
                                    "Restart required"
                                </span>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }}
                    </div>
                    <p class="text-xs text-slate-400 mt-1 leading-relaxed">{description}</p>
                </div>

                // Modified Badge
                {if is_modified {
                    view! {
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-cyan-500/20 text-cyan-300 border border-cyan-500/40">
                            "Customized"
                        </span>
                    }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }}
            </div>

            // INPUT SECTION ACCORDING TO OPTION TYPE
            <div class="pt-2">
                {match opt_type {
                    OptionType::Bool => {
                        let state = state.clone();
                        let k = key.clone();
                        let current_bool = setting.effective_value().as_bool().unwrap_or(false);

                        view! {
                            <div class="flex items-center space-x-3">
                                <button
                                    on:click=move |_| {
                                        state.save_setting(k.clone(), Some(serde_json::json!(!current_bool)));
                                    }
                                    role="switch"
                                    aria-checked=if current_bool { "true" } else { "false" }
                                    class=if current_bool {
                                        "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-cyan-500 transition-colors duration-200 ease-in-out focus:outline-none shadow-md shadow-cyan-500/20"
                                    } else {
                                        "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-slate-800 transition-colors duration-200 ease-in-out focus:outline-none"
                                    }
                                >
                                    <span
                                        class=if current_bool {
                                            "translate-x-5 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                                        } else {
                                            "translate-x-0 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-slate-400 shadow ring-0 transition duration-200 ease-in-out"
                                        }
                                    />
                                </button>
                                <span class="text-xs font-medium text-slate-300">
                                    {if current_bool { "Enabled" } else { "Disabled" }}
                                </span>
                            </div>
                        }.into_any()
                    }
                    OptionType::String => {
                        let state = state.clone();
                        let k = key.clone();
                        let current_str = setting.effective_value().as_str().unwrap_or("").to_string();

                        if !possible_values.is_empty() {
                            // Dropdown selector
                            view! {
                                <select
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        state.save_setting(k.clone(), Some(serde_json::json!(val)));
                                    }
                                    class="px-3 py-1.5 rounded-xl bg-slate-950 border border-slate-700 text-xs text-white focus:outline-none focus:ring-2 focus:ring-cyan-500"
                                >
                                    {possible_values.into_iter().map(|pv| {
                                        let val_str = pv.value.as_str().unwrap_or("").to_string();
                                        let is_selected = val_str == current_str;
                                        view! {
                                            <option value=val_str.clone() selected=is_selected>
                                                {pv.name}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                            }.into_any()
                        } else {
                            // Text input
                            let current_display = current_str.clone();
                            view! {
                                <div class="flex items-center space-x-2 max-w-md">
                                    <input
                                        type="text"
                                        prop:value=current_display
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            state.save_setting(k.clone(), Some(serde_json::json!(val)));
                                        }
                                        class="flex-1 px-3 py-1.5 rounded-xl bg-slate-950 border border-slate-700 text-xs text-white focus:outline-none focus:ring-2 focus:ring-cyan-500"
                                    />
                                    {if let Some(u) = unit {
                                        view! { <span class="text-xs text-slate-400">{u}</span> }.into_any()
                                    } else {
                                        view! { <span/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                    }
                    OptionType::Int => {
                        let state = state.clone();
                        let k = key.clone();
                        let current_num = setting.effective_value().as_i64().unwrap_or(0);

                        view! {
                            <div class="flex items-center space-x-2 max-w-xs">
                                <input
                                    type="number"
                                    prop:value=current_num.to_string()
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        if let Ok(num) = val.parse::<i64>() {
                                            state.save_setting(k.clone(), Some(serde_json::json!(num)));
                                        }
                                    }
                                    class="w-32 px-3 py-1.5 rounded-xl bg-slate-950 border border-slate-700 text-xs text-white font-mono focus:outline-none focus:ring-2 focus:ring-cyan-500"
                                />
                                {if let Some(u) = unit {
                                    view! { <span class="text-xs text-slate-400">{u}</span> }.into_any()
                                } else {
                                    view! { <span/> }.into_any()
                                }}
                            </div>
                        }.into_any()
                    }
                    OptionType::StringArray => {
                        let state = state.clone();
                        let k = key.clone();
                        let items: Vec<String> = setting.effective_value()
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();

                        let items_clone = items.clone();
                        let k_for_add = k.clone();
                        let state_for_add = state.clone();

                        view! {
                            <div class="space-y-2">
                                // Existing rules list
                                <div class="space-y-1.5">
                                    {items.into_iter().enumerate().map(|(idx, rule)| {
                                        let rule_text = rule.clone();
                                        let state_del = state.clone();
                                        let k_del = k.clone();
                                        let mut current_items = items_clone.clone();

                                        view! {
                                            <div class="flex items-center justify-between p-2 rounded-lg bg-slate-950/80 border border-slate-800 text-xs font-mono">
                                                <span class="text-slate-300">{rule_text}</span>
                                                <button
                                                    on:click=move |_| {
                                                        if idx < current_items.len() {
                                                            current_items.remove(idx);
                                                            state_del.save_setting(k_del.clone(), Some(serde_json::json!(current_items)));
                                                        }
                                                    }
                                                    title="Remove rule"
                                                    class="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                                                >
                                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                    </svg>
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>

                                // Add new rule input
                                <div class="flex items-center space-x-2 pt-1">
                                    <input
                                        type="text"
                                        placeholder="Add new rule (e.g. domain.com block)..."
                                        prop:value=move || new_rule_input.get()
                                        on:input=move |ev| set_new_rule_input.set(event_target_value(&ev))
                                        class="flex-1 px-3 py-1.5 rounded-xl bg-slate-950 border border-slate-700/80 text-xs text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-cyan-500"
                                    />
                                    <button
                                        on:click=move |_| {
                                            let text = new_rule_input.get();
                                            if !text.trim().is_empty() {
                                                let mut updated = items_clone.clone();
                                                updated.push(text.trim().to_string());
                                                state_for_add.save_setting(k_for_add.clone(), Some(serde_json::json!(updated)));
                                                set_new_rule_input.set(String::new());
                                            }
                                        }
                                        class="px-3 py-1.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-semibold text-xs transition-colors flex items-center space-x-1"
                                    >
                                        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                                        </svg>
                                        <span>"Add"</span>
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                    _ => view! { <div/> }.into_any(),
                }}
            </div>

            // "Reset to system default" footer button when customized
            {if is_modified {
                view! {
                    <div class="pt-2 border-t border-slate-800/60 flex justify-end">
                        <button
                            on:click=on_reset
                            class="text-xs text-slate-400 hover:text-amber-400 hover:underline flex items-center space-x-1.5 transition-colors"
                        >
                            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                            <span>"Reset to system default"</span>
                        </button>
                    </div>
                }.into_any()
            } else {
                view! { <span/> }.into_any()
            }}
        </div>
    }
}
