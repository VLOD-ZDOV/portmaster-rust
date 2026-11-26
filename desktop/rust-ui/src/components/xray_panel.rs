//! Dedicated Xray / VPN Control Panel & Interactive World Map matching Portmaster.

use std::collections::BTreeMap;

use leptos::prelude::*;
use pm_models::xray::XrayNode;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

use crate::i18n::t;
use crate::state::AppState;

/// Compact country [lat, lon] centroids for world map markers.
fn country_center(code: &str) -> Option<(f64, f64)> {
    let c = match code.to_ascii_uppercase().as_str() {
        "AD" => (42.54, 1.58), "AE" => (23.42, 53.85), "AL" => (41.15, 20.17), "AM" => (40.07, 45.04),
        "AR" => (-38.42, -63.62), "AT" => (47.52, 14.55), "AU" => (-25.27, 133.78), "AZ" => (40.14, 47.58),
        "BA" => (43.92, 17.68), "BE" => (50.50, 4.47), "BG" => (42.73, 25.49), "BH" => (26.07, 50.55),
        "BR" => (-14.24, -51.93), "BY" => (53.71, 27.95), "CA" => (56.13, -106.35), "CH" => (46.82, 8.23),
        "CL" => (-35.68, -71.54), "CN" => (35.86, 104.20), "CO" => (4.57, -74.30), "CR" => (9.75, -83.75),
        "CY" => (35.13, 33.43), "CZ" => (49.82, 15.47), "DE" => (51.17, 10.45), "DK" => (56.26, 9.50),
        "DO" => (18.74, -70.16), "DZ" => (28.03, 1.66), "EC" => (-1.83, -78.18), "EE" => (58.60, 25.01),
        "EG" => (26.82, 30.80), "ES" => (40.46, -3.75), "FI" => (61.92, 25.75), "FR" => (46.23, 2.21),
        "GB" | "UK" => (54.00, -2.00), "GE" => (42.32, 43.36), "GR" => (39.07, 21.82), "GT" => (15.78, -90.23),
        "HK" => (22.32, 114.17), "HR" => (45.10, 15.20), "HU" => (47.16, 19.50), "ID" => (-2.55, 118.02),
        "IE" => (53.41, -8.24), "IL" => (31.05, 34.85), "IN" => (20.59, 78.96), "IQ" => (33.22, 43.68),
        "IR" => (32.43, 53.69), "IS" => (64.96, -19.02), "IT" => (41.87, 12.57), "JP" => (36.20, 138.25),
        "KE" => (0.02, 37.91), "KG" => (41.20, 74.77), "KR" => (35.91, 127.77), "KZ" => (48.02, 67.88),
        "LT" => (55.17, 23.88), "LU" => (49.82, 6.13), "LV" => (56.95, 24.11), "MA" => (31.79, -7.09),
        "MD" => (47.41, 28.37), "MK" => (41.61, 21.75), "MN" => (46.86, 103.85), "MT" => (35.94, 14.38),
        "MX" => (23.63, -102.55), "MY" => (4.21, 101.98), "NG" => (9.08, 8.68), "NL" => (52.13, 5.29),
        "NO" => (60.47, 8.47), "NP" => (28.39, 84.12), "NZ" => (-40.90, 174.89), "PE" => (-9.19, -75.02),
        "PH" => (12.88, 121.77), "PK" => (30.38, 69.35), "PL" => (51.92, 19.15), "PT" => (39.40, -8.22),
        "PY" => (-23.44, -58.44), "RO" => (45.94, 24.97), "RS" => (44.02, 21.01), "RU" => (61.52, 105.32),
        "SA" => (23.89, 45.08), "SE" => (60.13, 18.64), "SG" => (1.35, 103.82), "SI" => (46.15, 14.99),
        "SK" => (48.67, 19.70), "TH" => (15.87, 100.99), "TR" => (38.96, 35.24), "TW" => (23.70, 120.96),
        "UA" => (48.38, 31.17), "US" => (37.09, -95.71), "UY" => (-32.52, -55.77), "UZ" => (41.38, 64.59),
        "VE" => (6.42, -66.59), "VN" => (14.06, 108.28), "ZA" => (-30.56, 22.94),
        _ => return None,
    };
    Some(c)
}

fn project_xy(lat: f64, lon: f64) -> (f64, f64) {
    let x = ((lon + 180.0) / 360.0) * 960.0;
    let y = ((84.0 - lat) / 145.0) * 420.0;
    (x.clamp(4.0, 956.0), y.clamp(4.0, 416.0))
}

#[derive(Deserialize)]
struct TopoTransform {
    scale: [f64; 2],
    translate: [f64; 2],
}

#[derive(Deserialize)]
struct TopoGeom {
    arcs: Vec<Vec<Vec<i64>>>,
}

#[derive(Deserialize)]
struct TopoObjects {
    land: TopoGeom,
}

#[derive(Deserialize)]
struct WorldTopoJson {
    objects: TopoObjects,
    arcs: Vec<Vec<[i64; 2]>>,
    transform: TopoTransform,
}

fn decode_arc(arcs: &[Vec<[i64; 2]>], tr: &TopoTransform, idx: i64) -> Vec<(f64, f64)> {
    let rev = idx < 0;
    let real_idx = if rev { (!idx) as usize } else { idx as usize };
    let Some(raw) = arcs.get(real_idx) else {
        return Vec::new();
    };
    let mut pts = Vec::with_capacity(raw.len());
    let mut qx = 0i64;
    let mut qy = 0i64;
    for pair in raw {
        qx += pair[0];
        qy += pair[1];
        let lon = (qx as f64) * tr.scale[0] + tr.translate[0];
        let lat = (qy as f64) * tr.scale[1] + tr.translate[1];
        pts.push((lat, lon));
    }
    if rev {
        pts.reverse();
    }
    pts
}

fn build_world_svg_path(topo: &WorldTopoJson) -> String {
    let mut out = String::with_capacity(120_000);
    for poly in &topo.objects.land.arcs {
        for ring in poly {
            let mut prev_lon: Option<f64> = None;
            let mut need_move = true;
            for &arc_idx in ring {
                for (lat, lon) in decode_arc(&topo.arcs, &topo.transform, arc_idx) {
                    if lat < -58.0 {
                        need_move = true;
                        prev_lon = Some(lon);
                        continue;
                    }
                    if let Some(pl) = prev_lon {
                        if (lon - pl).abs() > 100.0 {
                            need_move = true;
                        }
                    }
                    let (x, y) = project_xy(lat, lon);
                    use std::fmt::Write;
                    if need_move {
                        let _ = write!(out, "M{:.1} {:.1}", x, y);
                        need_move = false;
                    } else {
                        let _ = write!(out, "L{:.1} {:.1}", x, y);
                    }
                    prev_lon = Some(lon);
                }
            }
        }
    }
    out
}

#[component]
pub fn XrayPanel() -> impl IntoView {
    let state = expect_context::<AppState>();
    let xray_status = state.xray_status;
    let xray_nodes = state.xray_nodes;
    let xray_subs = state.xray_subscriptions;
    let is_busy = state.is_busy;
    let route_by_default = state.route_by_default;
    let settings = state.settings;
    let lang = state.lang;

    let (new_sub_url, set_new_sub_url) = signal(String::new());
    let (manual_links, set_manual_links) = signal(String::new());
    let (selected_country, set_selected_country) = signal(Option::<String>::None);
    let (map_path, set_map_path) = signal(String::new());

    // Fetch /assets/world-50m.json once when panel mounts
    spawn_local(async move {
        let url = format!("{}/assets/world-50m.json", crate::state::app_state::api_base_url());
        if let Ok(resp) = gloo_net::http::Request::get(&url).send().await {
            if let Ok(topo) = resp.json::<WorldTopoJson>().await {
                set_map_path.set(build_world_svg_path(&topo));
            }
        }
    });

    let firewall_rules = move || -> Vec<String> {
        settings
            .get()
            .into_iter()
            .find(|s| s.key == "spn/usagePolicy")
            .and_then(|s| s.effective_value().as_array().cloned())
            .map(|arr| arr.into_iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };
    let xray_rules = move || -> Vec<String> {
        settings
            .get()
            .into_iter()
            .find(|s| s.key == "xray/rules")
            .and_then(|s| s.effective_value().as_array().cloned())
            .map(|arr| arr.into_iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };
    let auto_connect = move || -> bool {
        settings
            .get()
            .into_iter()
            .find(|s| s.key == "xray/autoConnect")
            .and_then(|s| s.effective_value().as_bool())
            .unwrap_or(false)
    };

    let state_toggle = state.clone();
    let on_toggle_vpn = move |_| state_toggle.toggle_vpn();

    let state_default = state.clone();
    let on_toggle_default_route = move |_| state_default.toggle_route_by_default();

    let state_auto = state.clone();
    let on_toggle_auto_connect = move |_| {
        state_auto.update_setting("xray/autoConnect", serde_json::json!(!auto_connect()));
    };

    let state_reset = state.clone();
    let on_reset_apps = move |_| state_reset.reset_all_routing_overrides();

    let state_dl_bin = state.clone();
    let state_dl_geo = state.clone();

    view! {
        <div class="space-y-4">
            // 1. CONNECTION STATUS BANNER + CORE DOWNLOAD TOOLS
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-4 flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div class="flex items-center space-x-3.5">
                    <div class=move || {
                        if xray_status.get().connected {
                            "w-11 h-11 rounded bg-[#143d24] border border-[#1de966]/40 flex items-center justify-center text-[#1de966]"
                        } else {
                            "w-11 h-11 rounded bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-[#888888]"
                        }
                    }>
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                        </svg>
                    </div>

                    <div>
                        <div class="flex items-center space-x-2">
                            <h2 class="text-sm font-semibold text-white">
                                {move || {
                                    let s = xray_status.get();
                                    let l = lang.get();
                                    if s.connected {
                                        s.node_name.unwrap_or_else(|| t(l, "xray.connected").to_string())
                                    } else {
                                        t(l, "xray.offline").to_string()
                                    }
                                }}
                            </h2>
                            {move || {
                                let s = xray_status.get();
                                let l = lang.get();
                                if s.connected {
                                    view! {
                                        <span class="px-2 py-0.5 rounded-[2px] text-[10px] font-semibold bg-[#143d24] text-[#1de966] border border-[#1de966]/30">
                                            {t(l, "xray.connected")}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="px-2 py-0.5 rounded-[2px] text-[10px] font-semibold bg-[#222222] text-[#888888] border border-[#2c2c2c]">
                                            {t(l, "xray.offline")}
                                        </span>
                                    }.into_any()
                                }
                            }}
                        </div>
                        <p class="text-xs text-[#ababab] mt-0.5">
                            {move || {
                                let s = xray_status.get();
                                if s.connected {
                                    let ctry = s.exit_country.or(s.country).unwrap_or_default();
                                    let ip = s.exit_ip.unwrap_or_default();
                                    format!("Exit: {} {} • Inbound {}", ip, ctry, s.inbound_address.unwrap_or_else(|| "auto".into()))
                                } else if !s.binary_available {
                                    "xray-rust binary not found — click Download below".to_string()
                                } else {
                                    "Encrypted Xray tunnel idle (Direct mode)".to_string()
                                }
                            }}
                        </p>
                    </div>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    <button
                        on:click=move |_| state_dl_bin.download_xray_binary()
                        disabled=move || is_busy.get()
                        class="btn-pm !bg-[#222222] hover:!bg-[#2c2c2c] text-[#ababab] hover:text-white border border-[#2c2c2c]"
                    >
                        {move || t(lang.get(), "xray.download_bin")}
                    </button>
                    <button
                        on:click=move |_| state_dl_geo.download_xray_geodata()
                        disabled=move || is_busy.get()
                        class="btn-pm !bg-[#222222] hover:!bg-[#2c2c2c] text-[#4e97fa] border border-[#2c2c2c]"
                    >
                        {move || t(lang.get(), "xray.download_geo")}
                    </button>
                    <button
                        on:click=on_toggle_vpn
                        disabled=move || is_busy.get()
                        class=move || {
                            if is_busy.get() {
                                "btn-pm !bg-[#222222] text-[#888888] cursor-not-allowed border border-[#2c2c2c]"
                            } else if xray_status.get().connected {
                                "btn-pm !bg-[#3d1414] hover:!bg-[#811818] text-[#e01d1d] font-semibold border border-[#e01d1d]/40"
                            } else {
                                "btn-pm !bg-[#4e97fa] hover:!bg-[#3d86e8] text-white font-semibold"
                            }
                        }
                    >
                        {move || {
                            let l = lang.get();
                            if is_busy.get() { "..." }
                            else if xray_status.get().connected { t(l, "xray.disconnect") }
                            else { t(l, "xray.connect") }
                        }}
                    </button>
                </div>
            </div>

            // 2. INTERACTIVE WORLD MAP (Pure Rust TopoJSON -> SVG)
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3 space-y-2">
                <div class="flex items-center justify-between text-xs">
                    <span class="font-semibold text-white">{move || t(lang.get(), "xray.map_title")}</span>
                    <div class="flex items-center space-x-2">
                        {move || {
                            if let Some(c) = selected_country.get() {
                                view! {
                                    <button
                                        on:click=move |_| set_selected_country.set(None)
                                        class="px-2 py-0.5 rounded bg-[#2c2c2c] text-[#4e97fa] text-[10px] hover:text-white cursor-pointer"
                                    >
                                        {format!("Filter: {} ✕", c)}
                                    </button>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }
                        }}
                    </div>
                </div>

                <div class="w-full bg-[#121213] border border-[#2c2c2c] rounded overflow-hidden">
                    <svg viewBox="0 0 960 420" class="w-full h-56 md:h-64 block">
                        <rect width="960" height="420" fill="#121213" />
                        <path
                            d=move || map_path.get()
                            fill="#232528"
                            stroke="#34373d"
                            stroke-width="0.6"
                        />
                        {
                            let s_map = state.clone();
                            move || {
                                let nodes = xray_nodes.get();
                                let active_id = xray_status.get().node_id;
                                let is_conn = xray_status.get().connected;
                                let mut by_country: BTreeMap<String, Vec<XrayNode>> = BTreeMap::new();
                                for n in nodes {
                                    let cc = n.country_code.trim().to_ascii_uppercase();
                                    if !cc.is_empty() {
                                        by_country.entry(cc).or_default().push(n);
                                    }
                                }
                                by_country.into_iter().filter_map(|(cc, list)| {
                                    let (lat, lon) = country_center(&cc)?;
                                    let (cx, cy) = project_xy(lat, lon);
                                    let has_active = is_conn && list.iter().any(|n| active_id.as_deref() == Some(&n.id));
                                    let first_id = list[0].id.clone();
                                    let cc_label = format!("{} ({})", cc, list.len());
                                    let s_click = s_map.clone();
                                    let cc_filter = cc.clone();
                                    Some(view! {
                                        <g
                                            class="cursor-pointer"
                                            on:click=move |_| {
                                                set_selected_country.set(Some(cc_filter.clone()));
                                                s_click.select_node(first_id.clone());
                                            }
                                        >
                                            {if has_active {
                                                view! {
                                                    <circle cx=cx cy=cy r="12" fill="#1de966" opacity="0.22" />
                                                }.into_any()
                                            } else {
                                                view! { <g/> }.into_any()
                                            }}
                                            <circle
                                                cx=cx
                                                cy=cy
                                                r=if has_active { "6" } else { "4.5" }
                                                fill=if has_active { "#1de966" } else { "#4e97fa" }
                                                stroke="#121213"
                                                stroke-width="1.5"
                                            >
                                                <title>{cc_label.clone()}</title>
                                            </circle>
                                            <text
                                                x=cx
                                                y=cy - 8.0
                                                text-anchor="middle"
                                                fill=if has_active { "#1de966" } else { "#cbd5e1" }
                                                font-size="9"
                                                font-family="monospace"
                                            >
                                                {cc}
                                            </text>
                                        </g>
                                    })
                                }).collect_view()
                            }
                        }
                    </svg>
                </div>
            </div>

            // 3. Global VPN Routing & Auto-Connect Controls
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3.5 space-y-3">
                <div class="flex items-center justify-between gap-4">
                    <div class="flex items-center space-x-3">
                        <button
                            type="button"
                            on:click=on_toggle_default_route
                            class=move || {
                                if route_by_default.get() {
                                    "w-10 h-5 rounded-full bg-[#1de966] p-0.5 transition-colors cursor-pointer flex items-center"
                                } else {
                                    "w-10 h-5 rounded-full bg-[#343434] p-0.5 transition-colors cursor-pointer flex items-center"
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
                        <div>
                            <div class="text-xs font-semibold text-white">
                                {move || if lang.get() == crate::i18n::Lang::Ru {
                                    "Весь трафик через VPN (Xray) по умолчанию"
                                } else {
                                    "Route All Apps via VPN (Xray) by Default"
                                }}
                            </div>
                            <div class="text-[11px] text-[#888888]">
                                {move || if lang.get() == crate::i18n::Lang::Ru {
                                    "Когда ВЫКЛ, работают только приложения с галочкой Xray и (+) правила маршрутизации."
                                } else {
                                    "When OFF, traffic goes direct unless an app forces Always VPN or matches a (+) rule."
                                }}
                            </div>
                        </div>
                    </div>

                    <span class=move || if route_by_default.get() {
                        "px-2 py-0.5 rounded-[2px] text-xs font-semibold bg-[#143d24] text-[#1de966] border border-[#1de966]/30"
                    } else {
                        "px-2 py-0.5 rounded-[2px] text-xs font-semibold bg-[#222222] text-[#888888] border border-[#2c2c2c]"
                    }>
                        {move || if route_by_default.get() { "Default: VPN" } else { "Default: Direct" }}
                    </span>
                </div>

                <div class="flex flex-wrap items-center justify-between gap-4 pt-2 border-t border-[#262626]">
                    <div class="flex items-center space-x-3">
                        <button
                            type="button"
                            on:click=on_toggle_auto_connect
                            class=move || {
                                if auto_connect() {
                                    "w-10 h-5 rounded-full bg-[#4e97fa] p-0.5 transition-colors cursor-pointer flex items-center"
                                } else {
                                    "w-10 h-5 rounded-full bg-[#343434] p-0.5 transition-colors cursor-pointer flex items-center"
                                }
                            }
                        >
                            <span class=move || {
                                if auto_connect() {
                                    "w-4 h-4 rounded-full bg-white shadow-sm transform translate-x-5 transition-transform"
                                } else {
                                    "w-4 h-4 rounded-full bg-[#ababab] shadow-sm transform translate-x-0 transition-transform"
                                }
                            }></span>
                        </button>
                        <div class="text-xs font-semibold text-white">
                            {move || if lang.get() == crate::i18n::Lang::Ru {
                                "Автоподключение Xray при запуске"
                            } else {
                                "Auto-Connect Xray on Startup"
                            }}
                        </div>
                    </div>

                    <button
                        type="button"
                        on:click=on_reset_apps
                        class="btn-pm !bg-[#222222] text-[#cacaca] border border-[#2c2c2c] hover:text-white"
                    >
                        {move || t(lang.get(), "apps.reset_routing")}
                    </button>
                </div>
            </div>

            // 4. SUBSCRIPTIONS + MANUAL PROXY LINKS
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3.5 space-y-3">
                <div class="flex items-center justify-between">
                    <h3 class="text-xs font-semibold text-white">{move || t(lang.get(), "xray.subs")}</h3>
                    <span class="text-[11px] text-[#888888]">"VLESS, VMess, Trojan, Shadowsocks, Hysteria2, WireGuard"</span>
                </div>

                <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
                    <input
                        type="text"
                        placeholder="https://... subscription URL"
                        prop:value=move || new_sub_url.get()
                        on:input=move |ev| set_new_sub_url.set(event_target_value(&ev))
                        class="input-pm text-xs flex-1"
                    />
                    <button
                        on:click={
                            let s = state.clone();
                            move |_| {
                                let url = new_sub_url.get().trim().to_string();
                                if !url.is_empty() {
                                    s.add_xray_subscription(url);
                                    set_new_sub_url.set(String::new());
                                }
                            }
                        }
                        class="btn-pm !bg-[#4e97fa] hover:!bg-[#3d86e8] text-white whitespace-nowrap"
                    >
                        {move || t(lang.get(), "xray.add_sub")}
                    </button>
                </div>

                <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
                    <input
                        type="text"
                        placeholder="vless://... | trojan://... | wireguard://..."
                        prop:value=move || manual_links.get()
                        on:input=move |ev| set_manual_links.set(event_target_value(&ev))
                        class="input-pm text-xs flex-1"
                    />
                    <button
                        on:click={
                            let s = state.clone();
                            move |_| {
                                let l = manual_links.get().trim().to_string();
                                if !l.is_empty() {
                                    s.add_xray_manual_nodes(l);
                                    set_manual_links.set(String::new());
                                }
                            }
                        }
                        class="btn-pm !bg-[#222222] hover:!bg-[#2c2c2c] text-[#ababab] hover:text-white border border-[#2c2c2c] whitespace-nowrap"
                    >
                        {move || t(lang.get(), "xray.add_manual")}
                    </button>
                </div>

                // Saved subscriptions list
                {
                    let s_sub = state.clone();
                    move || {
                        let subs = xray_subs.get();
                        if subs.is_empty() {
                            return view! { <span/> }.into_any();
                        }
                        let sr = s_sub.clone();
                        let sd = s_sub.clone();
                        view! {
                            <div class="space-y-1 pt-2 border-t border-[#262626]">
                                {subs.into_iter().map(|sub| {
                                    let id_ref = sub.id.clone();
                                    let id_del = sub.id.clone();
                                    let sr2 = sr.clone();
                                    let sd2 = sd.clone();
                                    view! {
                                        <div class="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded bg-[#121213] border border-[#2c2c2c] text-xs">
                                            <div class="min-w-0">
                                                <div class="text-white font-medium truncate">{sub.name}</div>
                                                <div class="text-[10px] text-[#888888] font-mono truncate">{format!("{} ({} nodes)", sub.url, sub.node_count)}</div>
                                            </div>
                                            <div class="flex items-center gap-1.5 flex-shrink-0">
                                                <button
                                                    on:click=move |_| sr2.refresh_xray_subscription(id_ref.clone())
                                                    class="px-2 py-0.5 rounded bg-[#222222] text-[#4e97fa] hover:bg-[#2c2c2c] cursor-pointer text-[11px]"
                                                    title="Refresh"
                                                >
                                                    "↻"
                                                </button>
                                                <button
                                                    on:click=move |_| sd2.delete_xray_subscription(id_del.clone())
                                                    class="px-2 py-0.5 rounded bg-[#222222] text-[#e01d1d] hover:bg-[#3d1414] cursor-pointer text-[11px]"
                                                    title="Delete"
                                                >
                                                    "✕"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }
            </div>

            // 5. SERVER NODES SELECTOR
            <div class="space-y-2">
                <div class="flex items-center justify-between pb-1 border-b border-[#2c2c2c]">
                    <h3 class="text-xs font-semibold text-white">
                        {move || format!("{} ({})", t(lang.get(), "xray.nodes"), xray_nodes.get().len())}
                    </h3>
                    <span class="text-[11px] text-[#888888]">"Select node to route traffic"</span>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {move || {
                        let current_status = xray_status.get();
                        let current_id = current_status.node_id.clone();
                        let country_filter = selected_country.get();
                        let nodes: Vec<_> = xray_nodes
                            .get()
                            .into_iter()
                            .filter(|n| {
                                country_filter
                                    .as_deref()
                                    .map(|cf| n.country_code.eq_ignore_ascii_case(cf))
                                    .unwrap_or(true)
                            })
                            .collect();

                        nodes.into_iter().map(|node| {
                            let node_id = node.id.clone();
                            let is_active = current_id.as_deref() == Some(&node_id) && current_status.connected;
                            let state_clone = state.clone();
                            let node_id_connect = node.id.clone();
                            let ping_color = match node.ping_ms {
                                Some(ms) if ms < 100 => "text-[#1de966]",
                                Some(ms) if ms < 200 => "text-[#e9d81d]",
                                _ => "text-[#888888]",
                            };
                            view! {
                                <div class=if is_active {
                                    "bg-[#1b1b1b] border border-[#1de966]/60 rounded p-3 flex items-center justify-between gap-3 shadow-sm transition-all"
                                } else {
                                    "bg-[#1b1b1b] border border-[#2c2c2c] hover:border-[#343434] rounded p-3 flex items-center justify-between gap-3 transition-all"
                                }>
                                    <div class="flex items-center space-x-2.5 min-w-0">
                                        <div class="w-7 h-7 rounded bg-[#222222] border border-[#2c2c2c] flex items-center justify-center text-[10px] font-mono font-bold text-white flex-shrink-0">
                                            {node.country_code.clone()}
                                        </div>
                                        <div class="min-w-0">
                                            <div class="flex items-center space-x-2">
                                                <span class="text-xs font-medium text-white truncate">{node.name.clone()}</span>
                                                <span class="px-1.5 py-0.2 rounded text-[9px] font-mono bg-[#222222] text-[#888888] border border-[#2c2c2c] uppercase">
                                                    {node.protocol.clone()}
                                                </span>
                                            </div>
                                            <p class="text-[10px] text-[#888888] font-mono truncate">{format!("{}:{}", node.address, node.port)}</p>
                                        </div>
                                    </div>
                                    <div class="flex items-center space-x-2 flex-shrink-0">
                                        {node.ping_ms.map(|ms| view! {
                                            <span class=format!("text-xs font-mono {}", ping_color)>{format!("{}ms", ms)}</span>
                                        })}
                                        <button
                                            on:click=move |_| state_clone.select_node(node_id_connect.clone())
                                            class=if is_active {
                                                "btn-pm !bg-[#143d24] text-[#1de966] border border-[#1de966]/40 cursor-default"
                                            } else {
                                                "btn-pm !bg-[#222222] text-[#ababab] border border-[#2c2c2c] hover:text-white"
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

            // 6. ACTIVE ROUTING RULES SUMMARY
            <div class="bg-[#1b1b1b] border border-[#2c2c2c] rounded p-3.5 space-y-3">
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="text-xs font-semibold text-white">{move || t(lang.get(), "xray.rules")}</h3>
                        <p class="text-[11px] text-[#888888]">"spn/usagePolicy + xray/rules (configured in Settings -> VPN / Xray)"</p>
                    </div>
                </div>

                <div class="flex flex-wrap gap-1.5">
                    {move || {
                        let mut combined = Vec::new();
                        for r in firewall_rules() {
                            combined.push(("FW", r));
                        }
                        for r in xray_rules() {
                            combined.push(("Xray", r));
                        }
                        if combined.is_empty() {
                            view! {
                                <span class="text-[11px] text-[#666666]">"- LAN / - Localhost bypass active"</span>
                            }.into_any()
                        } else {
                            combined.into_iter().map(|(tag, rule)| {
                                let is_proxy = rule.trim_start().starts_with('+');
                                view! {
                                    <span class=if is_proxy {
                                        "px-2 py-0.5 rounded bg-[#222222] text-[#4e97fa] border border-[#2c2c2c] text-[11px] font-mono"
                                    } else {
                                        "px-2 py-0.5 rounded bg-[#222222] text-[#888888] border border-[#2c2c2c] text-[11px] font-mono"
                                    }>
                                        {format!("[{}] {}", tag, rule)}
                                    </span>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
