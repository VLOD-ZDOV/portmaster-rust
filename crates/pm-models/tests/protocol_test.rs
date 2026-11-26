use pm_models::*;
use serde_json::json;

#[test]
fn test_frame_get_roundtrip() {
    let req = PortApiRequest::get("req-1", "config:options/filter/defaultAction");
    let wire = req.to_wire();
    assert_eq!(wire, "req-1|get|config:options/filter/defaultAction");

    let parsed = PortApiFrame::parse(&wire).expect("parse frame");
    assert_eq!(parsed.id, "req-1");
    assert_eq!(parsed.msg_type, "get");
    assert_eq!(parsed.key.as_deref(), Some("config:options/filter/defaultAction"));
    assert_eq!(parsed.data, None);
}

#[test]
fn test_frame_query_sub_roundtrip() {
    let req = PortApiRequest::sub("sub-1", "core:profiles/");
    let wire = req.to_wire();
    assert_eq!(wire, "sub-1|sub|query core:profiles/");

    let parsed = PortApiFrame::parse(&wire).expect("parse frame");
    assert_eq!(parsed.id, "sub-1");
    assert_eq!(parsed.msg_type, "sub");
    assert_eq!(parsed.key.as_deref(), Some("core:profiles/"));
}

#[test]
fn test_frame_create_with_dsd_json() {
    let data = json!({
        "Name": "Firefox",
        "Source": "local"
    });
    let req = PortApiRequest::create("op-create", "core:profiles/local/firefox", data);
    let wire = req.to_wire();
    assert!(wire.starts_with("op-create|create|core:profiles/local/firefox|J{"));

    let parsed = PortApiFrame::parse(&wire).expect("parse");
    assert_eq!(parsed.id, "op-create");
    assert_eq!(parsed.msg_type, "create");
    assert_eq!(parsed.key.as_deref(), Some("core:profiles/local/firefox"));
    assert!(parsed.data.is_some());
    let val: serde_json::Value = parsed.parse_data().expect("json data");
    assert_eq!(val["Name"], "Firefox");
}

#[test]
fn test_frame_replies() {
    // OK reply with J-prefix
    let reply_raw = "op-1|ok|core:profiles/local/chrome|J{\"ID\":\"chrome\",\"Name\":\"Google Chrome\"}";
    let frame = PortApiFrame::parse(reply_raw).expect("parse ok");
    assert_eq!(frame.id, "op-1");
    assert_eq!(frame.msg_type, "ok");
    assert_eq!(frame.key.as_deref(), Some("core:profiles/local/chrome"));
    let profile: AppProfile = frame.parse_data().expect("parse profile");
    assert_eq!(profile.id, "chrome");
    assert_eq!(profile.name, "Google Chrome");

    // Success reply
    let reply_success = "op-2|success";
    let frame_success = PortApiFrame::parse(reply_success).expect("parse success");
    assert_eq!(frame_success.id, "op-2");
    assert_eq!(frame_success.msg_type, "success");

    // Done reply
    let reply_done = "op-3|done";
    let frame_done = PortApiFrame::parse(reply_done).expect("parse done");
    assert_eq!(frame_done.id, "op-3");
    assert_eq!(frame_done.msg_type, "done");

    // Error reply
    let reply_err = "op-4|error|profile not found";
    let frame_err = PortApiFrame::parse(reply_err).expect("parse err");
    assert_eq!(frame_err.id, "op-4");
    assert_eq!(frame_err.msg_type, "error");
    assert_eq!(frame_err.key.as_deref(), Some("profile not found"));
}

#[test]
fn test_app_profile_vpn_toggle() {
    let mut profile = AppProfile::new("telegram", "Telegram Desktop");
    assert!(!profile.route_via_vpn());

    profile.set_route_via_vpn(true);
    assert!(profile.route_via_vpn());

    profile.set_route_via_vpn(false);
    assert!(!profile.route_via_vpn());

    // Split Tunnel is a separate feature and must not be reported as VPN.
    profile.config = Some(json!({"splittun/use": true}));
    assert!(!profile.route_via_vpn());
    profile.set_route_via_vpn(true);
    assert!(profile.route_via_vpn());
    assert_eq!(profile.config.as_ref().unwrap()["splittun/use"], json!(true));
}

#[test]
fn test_xray_models() {
    let status = XrayStatus {
        enabled: true,
        connected: true,
        state: ProcessState::Running,
        node_id: Some("node-nl-01".to_string()),
        node_name: Some("Netherlands Premium #1".to_string()),
        traffic: Some(TrafficStatus {
            uplink: 1024 * 1024 * 5,
            downlink: 1024 * 1024 * 25,
        }),
        exit_ip: Some("198.51.100.1".to_string()),
        exit_country: Some("NL".to_string()),
        ..Default::default()
    };

    let serialized = serde_json::to_string(&status).expect("serialize");
    let deserialized: XrayStatus = serde_json::from_str(&serialized).expect("deserialize");
    assert_eq!(deserialized.state, ProcessState::Running);
    assert!(deserialized.connected);
    assert_eq!(deserialized.exit_country.as_deref(), Some("NL"));
    assert_eq!(deserialized.traffic.unwrap().formatted_downlink(), "25.0 MB");
}

#[test]
fn test_app_profile_three_state_overrides_and_subsystems() {
    let mut profile = AppProfile::new("app", "App");
    assert_eq!(profile.vpn_route_override(), None);
    assert_eq!(profile.split_tun_override(), None);

    profile.set_vpn_route_override(Some(true));
    profile.set_split_tun_override(Some(false));
    assert_eq!(profile.vpn_route_override(), Some(true));
    assert_eq!(profile.split_tun_override(), Some(false));

    profile.set_vpn_route_override(None);
    profile.set_split_tun_override(None);
    assert_eq!(profile.vpn_route_override(), None);
    assert_eq!(profile.split_tun_override(), None);
    assert!(profile.config.is_none());

    let spn_setting = Setting {
        key: "spn/use".to_string(),
        name: "Route".to_string(),
        description: String::new(),
        help: String::new(),
        opt_type: OptionType::Bool,
        value: None,
        default_value: json!(false),
        expertise_level: 0,
        release_level: 0,
        annotations: SettingAnnotations::default(),
        possible_values: vec![],
        requires_restart: false,
        meta: None,
    };
    assert_eq!(spn_setting.subsystem(), "xray");
}
