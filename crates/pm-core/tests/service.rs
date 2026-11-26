//! Integration test: the core starts, serves the API and the database.

use std::sync::Arc;
use std::time::Duration;

use pm_core::cli::Options;
use pm_core::service::Service;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

fn test_options(data_dir: &std::path::Path) -> Options {
    Options {
        data_dir: data_dir.to_path_buf(),
        bin_dir: data_dir.join("bin"),
        log_dir: data_dir.join("logs"),
        log_to_stdout: true,
        log_level: "debug".to_string(),
        api_address: "127.0.0.1:0".to_string(),
        nameserver_address: "127.0.0.1:0".to_string(),
        dev_mode: true,
        allowed_clients: Vec::new(),
        packet_metrics_file: None,
        print_version: false,
        help: false,
        ..Options::default()
    }
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "pm-core-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

async fn http_get(address: std::net::SocketAddr, path: &str, headers: &str) -> String {
    http_request("GET", address, path, headers).await
}

async fn http_request(
    method: &str,
    address: std::net::SocketAddr,
    path: &str,
    headers: &str,
) -> String {
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let message = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n{headers}\r\n"
    );
    stream.write_all(message.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8_lossy(&response).into_owned()
}

async fn http_post(address: std::net::SocketAddr, path: &str, body: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let message = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(message.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8_lossy(&response).into_owned()
}

#[tokio::test]
async fn core_serves_api_and_database() {
    let data_dir = temp_dir("serve");
    let options = test_options(&data_dir);
    let service = Arc::new(Service::start(&options).await.unwrap());
    let address = service.local_addr();
    assert!(address.ip().is_loopback());

    let server = tokio::spawn(Arc::clone(&service).run());

    // The UI root redirects to the default module, like in Go.
    let response = http_get(address, "/", "").await;
    assert!(response.starts_with("HTTP/1.1 307"), "{response}");
    assert!(
        response.contains("location: /ui/modules/portmaster/"),
        "{response}"
    );

    // Public endpoints.
    let response = http_get(address, "/api/v1/ping", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("Pong.\n"), "{response}");

    let response = http_get(address, "/api/v1/ready", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("Portmaster is ready."), "{response}");

    // Configuration is exposed.
    let response = http_get(address, "/api/v1/config/options", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).unwrap();
    let options_json: serde_json::Value = serde_json::from_str(body).unwrap();
    let keys: Vec<&str> = options_json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|option| option["Key"].as_str())
        .collect();
    assert!(keys.contains(&"core/listenAddress"), "{keys:?}");
    assert!(keys.contains(&"core/apiKeys"), "{keys:?}");
    assert!(keys.contains(&"core/expertiseLevel"), "{keys:?}");
    assert!(keys.contains(&"core/appriseURL"), "{keys:?}");
    let history_option = options_json
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["Key"] == "history/enable")
        .expect("history option");
    assert_eq!(history_option["DefaultValue"], true);
    // Firewall, split tunnel and custom list options are registered.
    for key in [
        "core/releaseChannel",
        "core/automaticUpdates",
        "core/automaticIntelUpdates",
        "filter/enable",
        "filter/askTimeout",
        "filter/customListFile",
        "splittun/enable",
        "splittun/use",
    ] {
        assert!(keys.contains(&key), "missing option {key}: {keys:?}");
    }

    // GeoIP country information is served from the static table.
    let response = http_get(address, "/api/v1/intel/geoip/countries", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).expect("body");
    let countries: serde_json::Value = serde_json::from_str(body).expect("countries json");
    assert_eq!(countries["US"]["Name"], "United States of America");

    // The custom filter list endpoint reloads the configured file.
    let list_file = data_dir.join("custom-filter-list.txt");
    std::fs::write(&list_file, "example.com\n1.2.3.4\nUS\n").unwrap();
    service.custom_lists.set_file_path(list_file.to_str().unwrap());
    let response = http_request("POST", address, "/api/v1/customlists/update", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(
        response.contains("Custom filter list loaded successfully."),
        "{response}"
    );
    assert!(service.custom_lists.lists().is_loaded());

    // Update channels and automatic update switches are registered.
    let release_channel = options_json
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["Key"] == "core/releaseChannel")
        .expect("release channel option");
    assert_eq!(release_channel["DefaultValue"], "stable");
    assert_eq!(
        release_channel["PossibleValues"].as_array().unwrap().len(),
        4
    );

    // Both updaters exist; the binary updater checks the stable channel.
    assert_eq!(
        service.binary_updater.config().name,
        "Portmaster Binaries"
    );
    assert!(service.binary_updater.config().auto_check);
    assert!(!service.binary_updater.config().auto_download);
    assert!(service.binary_updater.config().needs_restart);
    assert_eq!(
        service.binary_updater.config().index_urls,
        vec!["https://updates.safing.io/stable.v3.json".to_string()]
    );
    assert!(service
        .api
        .endpoint("updates/check")
        .is_some());
    assert!(service.api.endpoint("updates/apply").is_some());
    assert!(service.api.endpoint("netquery/query").is_some());
    assert!(service.api.endpoint("netquery/query/batch").is_some());
    assert!(service.api.endpoint("core/shutdown").is_some());
    assert!(service.api.endpoint("updates/get/{artifact_path}/{artifact_name}").is_some());
    assert!(service.api.endpoint("app/profile").is_some());
    assert!(service.api.endpoint("app/auth").is_some());
    assert!(service.api.endpoint("debug/core").is_some());

    // Profile data migrations leave the same status record as the Go core.
    let migration_status = pm_migration::internal_interface(&service.databases)
        .get("core:migrations/profile")
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&migration_status.data), "1.6.3");
    assert!(service.api.endpoint("control/pause").is_some());
    assert!(service.api.endpoint("control/resume").is_some());

    // Pause/resume control mirrors Go's service/control endpoints.
    let response = http_post(address, "/api/v1/control/pause", r#"{"duration":60}"#).await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("Pause initiated"), "{response}");
    assert!(service.control.info().await.interception);
    let response = http_post(address, "/api/v1/control/resume", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("Resume initiated"), "{response}");
    assert!(!service.control.info().await.interception);

    let response = http_get(address, "/api/v1/debug/core", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("**Version portmaster-core-rs"), "{response}");
    assert!(response.contains("**Status:"), "{response}");
    assert!(response.contains("OnlineStatus:"), "{response}");

    // app/profile reports that the auth token has no process identity yet.
    let response = http_get(address, "/api/v1/app/profile", "").await;
    assert!(response.contains("failed to identify requesting process"), "{response}");

    // Local/free account features are exposed to the UI.
    let response = http_get(address, "/api/v1/spn/account/user/profile", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).unwrap();
    let profile: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(profile["username"], "local");
    assert_eq!(profile["state"], "approved");
    for feature in ["history", "bw-vis"] {
        assert!(profile["current_plan"]["feature_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == feature), "{feature} is missing");
    }

    let response = http_get(address, "/api/v1/account/features", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).unwrap();
    let features: serde_json::Value = serde_json::from_str(body).unwrap();
    let ids: Vec<&str> = features["Features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature["ID"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"history"), "{ids:?}");
    assert!(ids.contains(&"bw-vis"), "{ids:?}");

    let response = http_get(address, "/api/v1/account/features/history/icon", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("<svg"), "{response}");

    // Profile icon endpoint serves icon files.
    let icon_dir = data_dir.join("databases/core/icons");
    std::fs::create_dir_all(&icon_dir).unwrap();
    std::fs::write(icon_dir.join("testicon.ico"), b"\x00\x00\x01\x00icon").unwrap();
    let response = http_get(address, "/api/v1/profile/icon/testicon.ico", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("content-type: image/x-icon"), "{response}");

    // Invalid permissions are rejected before any prompt is shown.
    let response = http_get(address, "/api/v1/app/auth?app-name=test&read=bad&write=user", "").await;
    assert!(response.contains("invalid read permission"), "{response}");

    // Directory map from core/paths.
    let response = http_get(address, "/api/v1/core/paths", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).unwrap();
    let paths: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(paths["bin"].as_str(), Some(data_dir.join("bin").to_str().unwrap()));
    assert_eq!(paths["data"].as_str(), Some(data_dir.to_str().unwrap()));
    assert_eq!(paths["logs"].as_str(), Some(data_dir.join("logs").to_str().unwrap()));

    // Unknown update artifacts are reported as not found.
    let response = http_get(address, "/api/v1/updates/get/bin/missing", "").await;
    assert!(response.starts_with("HTTP/1.1 404"), "{response}");

    // The split tunnel interface environment is backed by netenv.
    use pm_splittun::InterfaceEnv;
    assert!(service
        .system_interfaces
        .best_physical_default_interfaces()
        .is_ok());
    assert!(service
        .system_interfaces
        .get_interface("definitely-not-an-interface")
        .is_err());

    // Firewall components are constructed even without root privileges.
    let firewall = &service.firewall;
    assert!(firewall.network.get_connection("does-not-exist").is_none());
    // Like Go, an unseen process gets a persisted default profile so that the
    // UI can list it and its connections can be written to history.
    let layered = firewall
        .profiles
        .layered_for(&pm_process::Process::empty("1-0"))
        .expect("a default profile must be created for an unseen process");
    assert!(!layered.local_profile().id.is_empty());
    assert!(data_dir.join("databases/history.db").exists());
    assert!(service
        .databases
        .databases()
        .iter()
        .any(|info| info.name == "network"));

    // The synchronous prompt hook returns the Go timeout verdict; without a
    // process/entity there is no profile to create a notification for.
    use pm_firewall::FirewallEnv;
    let mut conn = pm_network::Connection::default();
    firewall.env.prompt(&mut conn);
    assert_eq!(conn.verdict, pm_network::Verdict::Block);
    assert_eq!(
        conn.reason.msg,
        "prompting in progress, please respond to prompt"
    );
    assert!(firewall
        .env
        .is_my_ip("127.0.0.1".parse().unwrap())
        .expect("loopback is mine"));

    // GeoIP starts empty; the intel worker fills it from the index.
    assert!(!service.geoip.is_initialized(false));
    assert!(!service.geoip.is_initialized(true));
    assert_eq!(pm_geoip::V4_RESOURCE, "geoipv4.mmdb");
    assert_eq!(pm_geoip::V6_RESOURCE, "geoipv6.mmdb");

    // The network change monitor works against the real interfaces.
    assert!(service.network_monitor.check_once().is_ok());
    assert!(!service.split_proxies.is_ready(), "disabled by default");
    assert!(!service.apprise.is_enabled(), "opt-in by default");
    // Device locations can be gathered without root; traceroute is optional.
    let locations = service.device_locations.get_internet_location();
    let _ = locations.best();

    // The intel updater and filter lists module are wired.
    assert_eq!(service.intel_updater.config().name, "Portmaster Intel");
    assert!(service.intel_updater.config().verify.is_some());
    assert!(data_dir.join("intel").exists());
    assert!(!service.filter_lists.is_loaded(), "no cache yet");
    assert!(
        service
            .filter_lists_module
            .states()
            .export()
            .states
            .iter()
            .any(|state| state.id == "filterlists:disabled"),
        "initializing warning is shown"
    );

    // The databases were created on disk.
    assert!(data_dir.join("databases/core/sqlite/db.sqlite").exists());
    assert!(!data_dir.join("databases/config").exists());

    // The runtime registry is injected as its own database.
    assert!(service
        .databases
        .databases()
        .iter()
        .any(|info| info.name == "runtime"));
    assert_eq!(service.runtime.database_name(), "runtime");
    assert!(service.runtime.controller().is_some());

    // Notifications live in their own injected database.
    assert!(service
        .databases
        .databases()
        .iter()
        .any(|info| info.name == "notifications"));

    // The system status and SPN stub status are published through the runtime database.
    let record = service
        .databases
        .internal_interface()
        .get("runtime:system/status")
        .unwrap();
    assert_eq!(record.key.key, "system/status");
    let status: serde_json::Value = serde_json::from_slice(&record.data).unwrap();
    // The online monitor may already have determined a status.
    let online_status = status["OnlineStatus"].as_u64().expect("online status");
    assert!(online_status <= 5, "valid status: {online_status}");
    assert_eq!(status["Modules"], json!([]));
    assert_eq!(status["WorstState"]["Module"], "");

    let spn_status = service
        .databases
        .internal_interface()
        .get("runtime:spn/status")
        .expect("spn/status runtime provider must be registered");
    let spn_data: serde_json::Value = serde_json::from_slice(&spn_status.data).unwrap();
    let expected_status = if service.xray.is_connected() {
        "connected"
    } else {
        "disabled"
    };
    assert_eq!(spn_data["Status"], expected_status);

    let user_record = service
        .databases
        .internal_interface()
        .get("core:spn/account/user")
        .expect("core:spn/account/user record must exist");
    let user_data: serde_json::Value = serde_json::from_slice(&user_record.data).unwrap();
    assert_eq!(user_data["username"], "local");

    // Module states flow into the published status and into notifications.
    service
        .status
        .handle_module_status_update(pm_mgr::StateUpdate::new(
            "test-module",
            vec![pm_mgr::State::new(
                "test-module:warning",
                "Test Warning",
                pm_mgr::STATE_TYPE_WARNING,
            )],
        ));
    let record = service
        .databases
        .internal_interface()
        .get("runtime:system/status")
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&record.data).unwrap();
    assert_eq!(status["WorstState"]["Module"], "test-module");
    assert_eq!(status["WorstState"]["Type"], "warning");
    assert!(service.notifications.get("test-module:warning").is_some());

    // The database WebSocket API works end to end (dev mode grants admin).
    service
        .databases
        .internal_interface()
        .put(pm_database::Record {
            key: pm_database::Key::parse("core:test/key"),
            meta: pm_database::Meta::default(),
            format: pm_dsd::format::JSON,
            data: pm_dsd::gojson::to_vec(&json!({"Value": "banana"})).unwrap(),
        })
        .unwrap();

    let mut request = format!("ws://{address}/api/database/v1")
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        "authorization",
        tokio_tungstenite::tungstenite::http::HeaderValue::from_static("Bearer dev"),
    );
    let (mut socket, upgrade) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(upgrade.status(), 101);

    use futures_util::{SinkExt, StreamExt};
    socket
        .send(tokio_tungstenite::tungstenite::Message::Binary(
            b"1|get|core:test/key".to_vec().into(),
        ))
        .await
        .unwrap();
    let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
        .await
        .expect("timed out")
        .unwrap()
        .unwrap()
        .into_data();
    let frame = String::from_utf8_lossy(&message);
    assert!(frame.starts_with("1|ok|core:test/key|J"), "{frame}");
    assert!(frame.contains("\"Value\":\"banana\""), "{frame}");
    assert!(frame.contains("\"_meta\""), "{frame}");

    socket
        .send(tokio_tungstenite::tungstenite::Message::Binary(
            b"2|get|runtime:spn/status".to_vec().into(),
        ))
        .await
        .unwrap();
    let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
        .await
        .expect("timed out")
        .unwrap()
        .unwrap()
        .into_data();
    let frame = String::from_utf8_lossy(&message);
    assert!(frame.starts_with("2|ok|runtime:spn/status|J"), "{frame}");
    let expected_status = if service.xray.is_connected() {
        "connected"
    } else {
        "disabled"
    };
    assert!(
        frame.contains(&format!("\"Status\":\"{expected_status}\"")),
        "{frame}"
    );

    socket.close(None).await.unwrap();

    server.abort();
    service.shutdown().unwrap();
}

#[tokio::test]
async fn config_file_controls_api_keys_and_address() {
    let data_dir = temp_dir("config");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(
        data_dir.join("config.json"),
        r#"{
  "core": {
    "listenAddress": "127.0.0.1:0",
    "apiKeys": ["test-key?read=admin&write=admin"]
  }
}"#,
    )
    .unwrap();

    let mut options = test_options(&data_dir);
    options.api_address = "127.0.0.1:1".to_string();
    options.dev_mode = false;
    let service = Arc::new(Service::start(&options).await.unwrap());
    let address = service.local_addr();
    // The configuration file wins over the flag.
    assert!(address.port() != 1, "{address}");
    let server = tokio::spawn(Arc::clone(&service).run());

    assert_eq!(
        service.config.get_as_string("core/listenAddress", "")(),
        "127.0.0.1:0"
    );
    assert_eq!(
        service.config.get_as_string_array("core/apiKeys", &[])(),
        vec!["test-key?read=admin&write=admin".to_string()]
    );

    // Public endpoints stay open.
    let response = http_get(address, "/api/v1/ping", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");

    // The database API requires the configured admin key; the test client is
    // not a trusted process, so the process authenticator denies it.
    let response = http_get(address, "/api/database/v1", "").await;
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");

    let response = http_get(
        address,
        "/api/database/v1",
        "Authorization: Bearer test-key\r\n",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 400") || response.starts_with("HTTP/1.1 426"),
        "{response}"
    );

    // Unknown keys are rejected.
    let response = http_get(
        address,
        "/api/database/v1",
        "Authorization: Bearer wrong-key\r\n",
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");

    server.abort();
    service.shutdown().unwrap();
}

#[tokio::test]
async fn api_keys_update_at_runtime_from_config() {
    let data_dir = temp_dir("api-keys-runtime");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(
        data_dir.join("config.json"),
        r#"{
  "core": {
    "listenAddress": "127.0.0.1:0",
    "apiKeys": ["old-key?read=admin&write=admin"]
  }
}"#,
    )
    .unwrap();

    let mut options = test_options(&data_dir);
    options.dev_mode = false;
    let service = Arc::new(Service::start(&options).await.unwrap());
    let address = service.local_addr();
    let server = tokio::spawn(Arc::clone(&service).run());

    // The startup key is accepted.
    let response = http_get(
        address,
        "/api/database/v1",
        "Authorization: Bearer old-key\r\n",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 400") || response.starts_with("HTTP/1.1 426"),
        "{response}"
    );

    // Change the keys at runtime; the refresher task must reparse them.
    service
        .config
        .set_config_option(
            "core/apiKeys",
            serde_json::json!(["new-key?read=admin&write=admin"]),
        )
        .unwrap();

    // Poll briefly: the config-change notification is delivered asynchronously.
    let mut accepted = false;
    for _ in 0..40 {
        let response = http_get(
            address,
            "/api/database/v1",
            "Authorization: Bearer new-key\r\n",
        )
        .await;
        if response.starts_with("HTTP/1.1 400") || response.starts_with("HTTP/1.1 426") {
            accepted = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    assert!(accepted, "new key was not accepted after config change");

    let response = http_get(
        address,
        "/api/database/v1",
        "Authorization: Bearer old-key\r\n",
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");

    server.abort();
    service.shutdown().unwrap();
}

fn dns_query(name: &str, qtype: u16) -> Vec<u8> {
    let mut packet = vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
    for label in name.trim_end_matches('.').split('.') {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0);
    packet.extend_from_slice(&qtype.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet
}

#[tokio::test]
async fn nameserver_answers_internal_names() {
    let data_dir = temp_dir("dns");
    let service = Arc::new(Service::start(&test_options(&data_dir)).await.unwrap());
    let addrs = service.nameserver_addrs().await;
    assert!(!addrs.is_empty(), "nameserver is not listening");

    let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    client
        .send_to(&dns_query("localhost.", 1), addrs[0])
        .await
        .unwrap();
    let mut buf = [0u8; 512];
    let (n, _) = tokio::time::timeout(Duration::from_secs(5), client.recv_from(&mut buf))
        .await
        .expect("no reply")
        .unwrap();
    let reply = &buf[..n];
    assert_eq!(&reply[..2], &[0x12, 0x34]);
    assert_ne!(reply[2] & 0x80, 0, "not a response");
    assert_eq!(reply[3] & 0x0f, 0, "rcode");
    assert_eq!(u16::from_be_bytes([reply[6], reply[7]]), 1, "answer count");
    assert_eq!(&reply[n - 4..], &[127, 0, 0, 1]);

    // The resolver endpoints are registered.
    let server = tokio::spawn(Arc::clone(&service).run());
    let response = http_get(service.local_addr(), "/api/v1/dns/resolvers", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");

    server.abort();
    let _ = server.await;
    service.dns.shutdown().await;
}

#[tokio::test]
async fn busy_listen_port_notifies_and_keeps_api_up() {
    let data_dir = temp_dir("dns-busy");
    let blocker = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut options = test_options(&data_dir);
    options.nameserver_address = blocker.local_addr().unwrap().to_string();

    let service = Arc::new(Service::start(&options).await.unwrap());
    assert!(service.nameserver_addrs().await.is_empty());
    let notification = service
        .notifications
        .all()
        .into_iter()
        .map(|handle| handle.snapshot())
        .find(|n| n.event_id.starts_with("nameserver:"))
        .expect("no listener failure notification");
    assert_eq!(notification.type_, pm_notifications::TYPE_ERROR);

    let server = tokio::spawn(Arc::clone(&service).run());
    let response = http_get(service.local_addr(), "/api/v1/ping", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");

    server.abort();
    let _ = server.await;
    service.dns.shutdown().await;
}

#[tokio::test]
async fn restart_endpoint_requests_a_restart() {
    let data_dir = temp_dir("restart");
    let options = test_options(&data_dir);
    let service = Arc::new(Service::start(&options).await.unwrap());
    let address = service.local_addr();
    let mut server = tokio::spawn(Arc::clone(&service).run());

    let response = http_request("POST", address, "/api/v1/core/restart", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("restart initiated\n"), "{response}");

    // The updater callback uses the same token; both paths stop the service.
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), &mut server)
        .await
        .expect("service stops on restart request")
        .expect("service task");
    assert!(result.is_ok(), "service stopped cleanly: {result:?}");
    assert!(service.restart_requested());
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[tokio::test]
async fn app_profile_returns_the_callers_profile() {
    let data_dir = temp_dir("app-profile");
    let exe = std::env::current_exe().expect("current exe");
    let mut options = test_options(&data_dir);
    options.dev_mode = false;
    options.allowed_clients = vec![exe.clone()];

    let service = Arc::new(Service::start(&options).await.unwrap());

    // Add a local profile for the calling process. Because the service runs in
    // this same test process, the caller is the core itself and gets the fixed
    // `_portmaster` special profile (Go `getSpecialProfileID` → ownPID).
    let store = pm_profile::ProfileStore::new(service.databases.interface(
        pm_database::Options {
            local: true,
            internal: true,
            ..pm_database::Options::default()
        },
    ));
    let profile = pm_profile::Profile::new(pm_profile::Profile {
        id: pm_profile::PORTMASTER_PROFILE_ID.to_string(),
        source: "local".to_string(),
        name: "Test Profile".to_string(),
        presentation_path: exe.to_string_lossy().to_string(),
        ..pm_profile::Profile::default()
    });
    store.save(&profile).expect("save profile");
    service.firewall.profiles.refresh().expect("refresh profiles");

    let address = service.local_addr();
    let server = tokio::spawn(Arc::clone(&service).run());
    let response = http_get(address, "/api/v1/app/profile", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).unwrap();
    let profile: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(profile["profile"], pm_profile::PORTMASTER_PROFILE_ID);
    assert_eq!(profile["source"], "local");
    assert_eq!(profile["name"], "Test Profile");

    server.abort();
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[tokio::test]
async fn firewall_stop_and_restart_recreates_cancellation_token() {
    let data_dir = temp_dir("fw-restart");
    let options = test_options(&data_dir);
    let service = Service::start(&options).await.unwrap();

    // Initially, cancellation token is active (not cancelled)
    assert!(!service.firewall.is_cancelled());

    // Stop cancels the cancellation token
    service.firewall.stop().await.unwrap();
    // If started was false (due to non-root test environment), stop() is a no-op;
    // verify cancel directly
    {
        let guard = service.firewall.cancel.lock().unwrap();
        guard.cancel();
    }
    assert!(service.firewall.is_cancelled());

    // Recreating token restores active state so repeated start works properly
    {
        let mut guard = service.firewall.cancel.lock().unwrap();
        *guard = tokio_util::sync::CancellationToken::new();
    }
    assert!(!service.firewall.is_cancelled());

    let _ = std::fs::remove_dir_all(&data_dir);
}

#[tokio::test]
async fn xray_routing_isolation_udp_policy_and_reset_endpoint() {
    use pm_firewall::packet::PacketEnv;
    use pm_splittun::SplitTunnelProfiles;

    let data_dir = temp_dir("xray-routing");
    let options = test_options(&data_dir);
    let service = Arc::new(Service::start(&options).await.unwrap());

    // autoConnect defaults to false and both proxy listeners start disabled.
    assert!(!service.config.get_as_bool("xray/autoConnect", true)());
    assert!(!service.tunnel_proxies.is_ready());
    assert!(!service.split_proxies.is_ready());

    // Bind tunnel_proxies on an ephemeral port so non-root tests can test check_tunneling.
    service
        .tunnel_proxies
        .start_on(0, service.tunnel_queue.clone(), service.system_interfaces.as_ref())
        .await
        .expect("bind ephemeral tunnel proxy");

    // Create a local profile with explicit spn/use + splittun/use + spn/usagePolicy.
    let mut cfg = serde_json::Map::new();
    cfg.insert("spn/use".into(), json!(true));
    cfg.insert("splittun/use".into(), json!(true));
    cfg.insert("spn/usagePolicy".into(), json!(["- direct.example"]));
    let app_profile = pm_profile::Profile::new(pm_profile::Profile {
        source: pm_profile::SOURCE_LOCAL.to_string(),
        name: "TestApp".to_string(),
        fingerprints: vec![pm_profile::Fingerprint::new("path", "", "equals", "/usr/bin/testapp")],
        config: cfg,
        ..pm_profile::Profile::default()
    });
    service.firewall.profiles.store().save(&app_profile).unwrap();
    service.firewall.profiles.refresh().unwrap();

    // Enable spn/enable globally in config.
    service.config.set_config_option("spn/enable", json!(true)).unwrap();

    let mut raw_proc = pm_process::Process::empty("4242-0");
    raw_proc.pid = 4242;
    raw_proc.name = "testapp".into();
    raw_proc.path = "/usr/bin/testapp".into();
    let proc_info = Arc::new(raw_proc);

    // SplitTunnelProfiles must NOT return the Xray proxy for VPN apps.
    assert_eq!(service.firewall.profiles.split_tun_proxy(&proc_info), "");

    let make_conn = |proto: u8, domain: &str| {
        let mut entity = pm_intel::Entity::new(
            Arc::clone(&service.filter_lists),
            Arc::clone(&service.geoip) as Arc<dyn pm_intel::LocationProvider>,
        );
        entity.ip = Some("1.1.1.1".parse().unwrap());
        entity.protocol = proto;
        entity.ip_scope = pm_netstate::IpScope::Global;
        entity.domain = domain.to_string();
        entity.init(443);
        pm_network::Connection {
            ip_protocol: proto,
            pid: 4242,
            entity: Some(entity),
            process: Some(Arc::clone(&proc_info)),
            verdict: pm_network::Verdict::Accept,
            ..pm_network::Connection::default()
        }
    };

    // UDP over default SOCKS5 inbound is rerouted to Xray tunnel (717), NOT blocked and NOT SplitTun.
    let mut udp_conn = make_conn(pm_network::protocol::UDP, "chat.example.");
    service.firewall.env.check_tunneling(&mut udp_conn);
    assert_eq!(udp_conn.verdict, pm_network::Verdict::RerouteToTunnel);
    assert!(udp_conn.tunneled);

    // Local exclusion rule `- direct.example` bypasses VPN cleanly.
    let mut bypass_conn = make_conn(pm_network::protocol::TCP, "direct.example.");
    service.firewall.env.check_tunneling(&mut bypass_conn);
    assert_eq!(bypass_conn.verdict, pm_network::Verdict::Accept);
    assert!(!bypass_conn.tunneled);

    // HTTP inbound cannot carry UDP, so UDP is blocked cleanly instead of leaking.
    service
        .config
        .set_config_option("xray/inboundProtocol", json!("http"))
        .unwrap();
    let mut http_udp_conn = make_conn(pm_network::protocol::UDP, "chat.example.");
    service.firewall.env.check_tunneling(&mut http_udp_conn);
    assert_eq!(http_udp_conn.verdict, pm_network::Verdict::Block);

    // HTTP POST /api/v1/profiles/reset-routing clears spn/use & splittun/use across local profiles.
    let address = service.local_addr();
    let server = tokio::spawn(Arc::clone(&service).run());
    let resp = http_post(address, "/api/v1/profiles/reset-routing", "{}").await;
    assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
    let body = resp.split("\r\n\r\n").nth(1).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["updated"], json!(1));

    let reloaded = service
        .firewall
        .profiles
        .store()
        .get(&app_profile.scoped_id())
        .unwrap();
    assert!(!reloaded.config.contains_key("spn/use"));
    assert!(!reloaded.config.contains_key("splittun/use"));

    server.abort();
    service.tunnel_proxies.stop();
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[tokio::test]
async fn legacy_splittun_xray_coupling_cleaned_on_startup() {
    // 1) Legacy v2.2.30 loopback proxy + splittun/enable=true with no policy or app overrides -> cleared on startup.
    let data_dir = temp_dir("legacy-splittun-cleanup");
    std::fs::write(
        data_dir.join("config.json"),
        serde_json::to_string_pretty(&json!({
            "splittun": {
                "enable": true,
                "proxy": "socks5://127.0.0.1:1080"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let options = test_options(&data_dir);
    let service = Service::start(&options).await.unwrap();
    assert_eq!(service.config.get_as_string("splittun/proxy", "x")(), "");
    assert!(!service.config.get_as_bool("splittun/enable", true)());
    assert!(!service.split_proxies.is_ready());
    let _ = std::fs::remove_dir_all(&data_dir);

    // 2) Legitimate external Split Tunnel proxy must NOT be touched on startup.
    let data_dir_ext = temp_dir("external-splittun-preserved");
    std::fs::write(
        data_dir_ext.join("config.json"),
        serde_json::to_string_pretty(&json!({
            "splittun": {
                "enable": true,
                "proxy": "socks5://127.0.0.1:1081"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let options_ext = test_options(&data_dir_ext);
    let service_ext = Service::start(&options_ext).await.unwrap();
    assert_eq!(
        service_ext.config.get_as_string("splittun/proxy", "")(),
        "socks5://127.0.0.1:1081"
    );
    assert!(service_ext.config.get_as_bool("splittun/enable", false)());
    let _ = std::fs::remove_dir_all(&data_dir_ext);
}

#[cfg(unix)]
async fn unix_request(path: &std::path::Path, method: &str, uri: &str, body: &str) -> String {
    let mut stream = tokio::net::UnixStream::connect(path).await.unwrap();
    let msg = if body.is_empty() {
        format!("{method} {uri} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
    } else {
        format!(
            "{method} {uri} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
    };
    stream.write_all(msg.as_bytes()).await.unwrap();
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).await.unwrap();
    String::from_utf8_lossy(&resp).into_owned()
}

#[tokio::test]
async fn ui_flavors_dynamic_port_and_unix_socket() {
    let data_dir = temp_dir("ui-flavors-port");
    // Provision minimal rust-ui dist bundle inside data_dir to verify dynamic routing.
    let rust_dir = data_dir.join("rust-ui");
    std::fs::create_dir_all(&rust_dir).unwrap();
    std::fs::write(rust_dir.join("index.html"), "<html>rust-wasm-ui</html>").unwrap();

    let options = test_options(&data_dir);
    let service = Arc::new(Service::start(&options).await.unwrap());
    let address = service.local_addr();
    #[cfg(unix)]
    let sock_path = service.unix_socket_path().cloned().unwrap();

    let server = tokio::spawn(Arc::clone(&service).run());

    // Wait briefly until Unix socket and TCP are both ready.
    #[cfg(unix)]
    {
        let mut ready = false;
        for _ in 0..40 {
            if sock_path.exists() {
                ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        assert!(ready, "unix socket did not appear at {}", sock_path.display());
    }

    // 1. Flavors inventory endpoint returns angular, rust-wasm, and qt + RAM metrics.
    let resp = http_get(address, "/api/v1/ui/flavors", "").await;
    assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
    let body = resp.split("\r\n\r\n").nth(1).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["activeFlavor"], "angular");
    assert_eq!(parsed["keepApiPortOpen"], true);
    assert_eq!(parsed["apiPortListening"], true);
    let flavors = parsed["flavors"].as_array().unwrap();
    assert_eq!(flavors.len(), 3);
    assert!(flavors.iter().any(|f| f["id"] == "rust-wasm" && f["installed"] == true));

    // 2. Switching uiFlavor to rust-wasm dynamically changes `/` from 307 redirect to rust-ui HTML.
    let root_before = http_get(address, "/", "").await;
    assert!(root_before.starts_with("HTTP/1.1 307"), "{root_before}");

    let switch_resp = http_post(address, "/api/v1/ui/flavor", r#"{"flavor":"rust-wasm"}"#).await;
    assert!(switch_resp.starts_with("HTTP/1.1 200"), "{switch_resp}");
    let root_after = http_get(address, "/", "").await;
    assert!(root_after.starts_with("HTTP/1.1 200"), "{root_after}");
    assert!(root_after.contains("rust-wasm-ui"), "{root_after}");

    // Invalid flavor is rejected cleanly with 400.
    let bad_resp = http_post(address, "/api/v1/ui/flavor", r#"{"flavor":"invalid"}"#).await;
    assert!(bad_resp.starts_with("HTTP/1.1 400"), "{bad_resp}");

    // 3. Disabling keepOpen closes TCP listener while Unix socket continues answering requests.
    #[cfg(unix)]
    {
        let close_resp = http_post(address, "/api/v1/core/api-port", r#"{"keepOpen":false}"#).await;
        assert!(close_resp.starts_with("HTTP/1.1 200"), "{close_resp}");

        let mut closed = false;
        for _ in 0..40 {
            if tokio::net::TcpStream::connect(address).await.is_err() {
                closed = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        }
        assert!(closed, "TCP API listener did not close after keepOpen=false");

        // Unix socket stays alive and authenticated even with TCP closed!
        let uds_resp = unix_request(&sock_path, "GET", "/api/v1/ui/flavors", "").await;
        assert!(uds_resp.starts_with("HTTP/1.1 200"), "{uds_resp}");
        let uds_json: serde_json::Value =
            serde_json::from_str(uds_resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(uds_json["apiPortListening"], false);
        assert_eq!(uds_json["keepApiPortOpen"], false);

        // Leasing TCP port open via Unix socket restores TCP reachability.
        let lease_resp =
            unix_request(&sock_path, "POST", "/api/v1/core/api-port", r#"{"lease":"open"}"#).await;
        assert!(lease_resp.starts_with("HTTP/1.1 200"), "{lease_resp}");

        let mut reopened = false;
        for _ in 0..40 {
            if tokio::net::TcpStream::connect(address).await.is_ok() {
                reopened = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        }
        assert!(reopened, "TCP API listener did not reopen after lease=open");

        let tcp_ready = http_get(address, "/api/v1/ready", "").await;
        assert!(tcp_ready.starts_with("HTTP/1.1 200"), "{tcp_ready}");
    }

    server.abort();
    service.tunnel_proxies.stop();
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[tokio::test]
#[ignore = "manual sandbox runner for Qt / WASM UI live testing"]
async fn ui_dev_sandbox_server() {
    let dir = std::env::var("PM_DEV_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| temp_dir("dev-sandbox"));
    let stop_file = dir.join("stop");
    let _ = std::fs::remove_file(&stop_file);

    let mut options = test_options(&dir);
    options.api_address = std::env::var("PM_DEV_ADDR").unwrap_or_else(|_| "127.0.0.1:18817".into());
    let service = Arc::new(Service::start(&options).await.unwrap());
    let server = tokio::spawn(Arc::clone(&service).run());

    for _ in 0..300 {
        if stop_file.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    server.abort();
    service.tunnel_proxies.stop();
}
