//! Dynamic UI serving (Angular zip, Rust Leptos WASM, pilot compatibility),
//! toggleable TCP listener controller, and local Unix Domain Socket server.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::extract::{ConnectInfo, Extension, Path as UrlPath, State};
#[cfg(unix)]
use axum::extract::Request as AxumRequest;
#[cfg(unix)]
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::Router;
use http::{header, HeaderValue, StatusCode};
use tokio_util::sync::CancellationToken;

use pm_api::Api;
use pm_config::Config;

use crate::service::CFG_UI_FLAVOR;

/// Legacy pilot UI directory name under `<data-dir>`.
pub const PILOT_ASSET_DIR: &str = "pilot-web";
/// Full Rust WASM UI directory name under `<data-dir>`.
pub const RUST_UI_ASSET_DIR: &str = "rust-ui";

#[cfg(windows)]
const QT_BIN_NAME: &str = "portmaster-qt.exe";
#[cfg(not(windows))]
const QT_BIN_NAME: &str = "portmaster-qt";

/// Shared state passed to UI route handlers so changes to `core/uiFlavor` or
/// newly installed UI bundles take effect immediately without restart.
#[derive(Clone)]
pub struct UiState {
    pub api: Arc<Api>,
    pub config: Arc<Config>,
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
}

/// Request extension kept exported for backwards-compatibility with tests/callers.
#[derive(Clone, Debug)]
pub struct PilotDir(pub PathBuf);

/// Finds the active Rust WASM asset directory, preferring `<data-dir>/rust-ui`,
/// `<bin-dir>/rust-ui`, system installs, and repo `desktop/rust-ui/dist`, falling
/// back to legacy `pilot-web` locations only when `rust-ui` is absent.
pub fn find_rust_ui_dir(data_dir: &Path, bin_dir: &Path) -> PathBuf {
    let rust_candidates = [
        data_dir.join(RUST_UI_ASSET_DIR),
        bin_dir.join(RUST_UI_ASSET_DIR),
        PathBuf::from("/usr/lib/portmaster").join(RUST_UI_ASSET_DIR),
        PathBuf::from("desktop/rust-ui/dist"),
        PathBuf::from("../desktop/rust-ui/dist"),
    ];
    if let Some(found) = rust_candidates
        .iter()
        .find(|p| p.is_dir() && p.join("index.html").is_file())
    {
        return found.clone();
    }

    let pilot_candidates = [
        data_dir.join(PILOT_ASSET_DIR),
        bin_dir.join(PILOT_ASSET_DIR),
        PathBuf::from("/usr/lib/portmaster").join(PILOT_ASSET_DIR),
        PathBuf::from("pilot/leptos/web"),
        PathBuf::from("../pilot/leptos/web"),
    ];
    if let Some(found) = pilot_candidates
        .iter()
        .find(|p| p.is_dir() && p.join("index.html").is_file())
    {
        return found.clone();
    }

    data_dir.join(RUST_UI_ASSET_DIR)
}

/// Finds an installed native Qt desktop GUI binary (`portmaster-qt`), if available.
pub fn find_qt_binary(bin_dir: &Path, data_dir: &Path) -> Option<PathBuf> {
    let mut candidates = vec![bin_dir.join(QT_BIN_NAME), data_dir.join(QT_BIN_NAME)];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(QT_BIN_NAME));
        }
    }
    #[cfg(not(windows))]
    {
        candidates.push(PathBuf::from("/usr/lib/portmaster/portmaster-qt"));
        candidates.push(PathBuf::from("/opt/safing/portmaster/portmaster-qt"));
        candidates.push(PathBuf::from("/usr/local/bin/portmaster-qt"));
        candidates.push(PathBuf::from("/usr/bin/portmaster-qt"));
    }
    candidates.push(PathBuf::from("desktop/qt-ui/build").join(QT_BIN_NAME));
    candidates.push(PathBuf::from("../desktop/qt-ui/build").join(QT_BIN_NAME));
    candidates.push(PathBuf::from("/tmp/pm-qt-build").join(QT_BIN_NAME));

    candidates.into_iter().find(|p| p.is_file())
}

/// Default local Unix domain socket path for IPC without an open TCP port.
#[cfg(unix)]
pub fn default_unix_socket_path(data_dir: &Path) -> PathBuf {
    if let Ok(custom) = std::env::var("PORTMASTER_API_SOCKET") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if nix::unistd::Uid::effective().is_root() {
        PathBuf::from("/run/portmaster/api.sock")
    } else {
        data_dir.join("api.sock")
    }
}

/// Normalizes a requested URL path into safe relative path segments.
///
/// Rejects absolute paths (both Unix and Windows style), any `..` component and
/// drive-letter prefixes. An empty path maps to `index.html`.
fn clean_pilot_path(requested: &str) -> Option<PathBuf> {
    if requested.starts_with('/') || requested.starts_with('\\') {
        return None;
    }

    let mut path = PathBuf::new();
    let mut empty = true;
    for segment in requested.split(['/', '\\']) {
        match segment {
            "" | "." => {}
            ".." => return None,
            other => {
                if other.contains(':') {
                    return None;
                }
                path.push(other);
                empty = false;
            }
        }
    }
    if empty {
        path.push("index.html");
    }
    Some(path)
}

/// Resolves a requested path to a canonical file inside `dir`.
fn resolve_pilot_file(dir: &Path, requested: &str) -> Option<(PathBuf, String)> {
    let relative = clean_pilot_path(requested)?;
    let root = std::fs::canonicalize(dir).ok()?;

    let candidate = std::fs::canonicalize(root.join(&relative)).ok()?;
    if !candidate.starts_with(&root) {
        return None;
    }

    let file = if candidate.is_dir() {
        std::fs::canonicalize(candidate.join("index.html")).ok()?
    } else {
        candidate
    };
    if !file.starts_with(&root) || !file.is_file() {
        return None;
    }

    let display = file.display().to_string();
    Some((file, display))
}

fn with_security_headers(api: &Api, mut response: AxumResponse) -> AxumResponse {
    api.apply_security_headers(response.headers_mut());
    response
}

async fn rust_ui_response(state: &UiState, requested: &str) -> AxumResponse {
    let dir = find_rust_ui_dir(&state.data_dir, &state.bin_dir);
    if !dir.is_dir() {
        return with_security_headers(
            &state.api,
            (StatusCode::NOT_FOUND, "rust ui is not installed\n").into_response(),
        );
    }

    let Some((path, name)) = resolve_pilot_file(&dir, requested) else {
        return with_security_headers(
            &state.api,
            (StatusCode::NOT_FOUND, "rust ui resource not found\n").into_response(),
        );
    };

    let data = match tokio::fs::read(&path).await {
        Ok(data) => data,
        Err(error) => {
            tracing::debug!(%error, path = %path.display(), "ui: failed to read rust-ui asset");
            return with_security_headers(
                &state.api,
                (StatusCode::NOT_FOUND, "rust ui resource not found\n").into_response(),
            );
        }
    };

    let content_type = pm_api::ui::content_type(&name);
    let mut response = AxumResponse::new(Body::from(data));
    *response.status_mut() = StatusCode::OK;
    if let Ok(value) = HeaderValue::from_str(content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    response
        .headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    with_security_headers(&state.api, response)
}

async fn serve_rust_index(Extension(state): Extension<UiState>) -> AxumResponse {
    rust_ui_response(&state, "").await
}

async fn serve_rust_file(
    Extension(state): Extension<UiState>,
    UrlPath(path): UrlPath<String>,
) -> AxumResponse {
    rust_ui_response(&state, &path).await
}

async fn serve_root(Extension(state): Extension<UiState>) -> AxumResponse {
    let env_rust = matches!(
        std::env::var("PORTMASTER_UI_TYPE").as_deref(),
        Ok("rust") | Ok("rust-wasm")
    );
    let cfg_flavor = state.config.get_as_string(CFG_UI_FLAVOR, "qt")();
    if env_rust || cfg_flavor == "rust-wasm" {
        let dir = find_rust_ui_dir(&state.data_dir, &state.bin_dir);
        if dir.is_dir() && dir.join("index.html").is_file() {
            return rust_ui_response(&state, "").await;
        }
    }
    pm_api::ui::redirect_root().await
}

async fn serve_legacy_pilot_js(Extension(state): Extension<UiState>) -> AxumResponse {
    rust_ui_response(&state, "pm_leptos_pilot.js").await
}

async fn serve_legacy_pilot_wasm(Extension(state): Extension<UiState>) -> AxumResponse {
    rust_ui_response(&state, "pm_leptos_pilot_bg.wasm").await
}

/// Builds the full HTTP/WebSocket API + Web UI router.
pub fn build_router(
    api: Arc<Api>,
    config: Arc<Config>,
    data_dir: impl AsRef<Path>,
    bin_dir: impl AsRef<Path>,
) -> Router {
    let ui_state = UiState {
        api: Arc::clone(&api),
        config,
        data_dir: data_dir.as_ref().to_path_buf(),
        bin_dir: bin_dir.as_ref().to_path_buf(),
    };

    Router::new()
        .route(
            pm_api::database_ws::DB_API_PATH,
            axum::routing::get(pm_api::database_ws::database_ws_upgrade),
        )
        .route(
            "/v1/database/ws",
            axum::routing::get(pm_api::database_ws::database_ws_upgrade),
        )
        .route("/assets/{*path}", axum::routing::get(pm_api::ui::serve_assets))
        .route(
            "/ui/modules/{module}",
            axum::routing::get(pm_api::ui::redirect_module),
        )
        .route(
            "/ui/modules/{module}/",
            axum::routing::get(pm_api::ui::serve_module_index),
        )
        .route(
            "/ui/modules/{module}/{*path}",
            axum::routing::get(pm_api::ui::serve_module_file),
        )
        .route("/", axum::routing::get(serve_root))
        .route("/rust", axum::routing::get(serve_rust_index))
        .route("/rust/", axum::routing::get(serve_rust_index))
        .route("/rust/{*path}", axum::routing::get(serve_rust_file))
        .route("/pilot", axum::routing::get(serve_rust_index))
        .route("/pilot/", axum::routing::get(serve_rust_index))
        .route("/pilot/{*path}", axum::routing::get(serve_rust_file))
        .route("/pm_leptos_pilot.js", axum::routing::get(serve_legacy_pilot_js))
        .route(
            "/pm_leptos_pilot_bg.wasm",
            axum::routing::get(serve_legacy_pilot_wasm),
        )
        .layer(Extension(ui_state))
        .fallback(axum::routing::any(handle_axum))
        .with_state(api)
}

/// Coordinates whether the TCP `:817` listener is continuously open or closed
/// unless temporarily leased open for a web UI session.
pub struct ApiPortController {
    listen_addr: Mutex<std::net::SocketAddr>,
    keep_open: AtomicBool,
    web_lease: AtomicBool,
    listening: AtomicBool,
    notify: tokio::sync::Notify,
}

impl ApiPortController {
    pub fn new(listen_addr: std::net::SocketAddr, keep_open: bool) -> Self {
        Self {
            listen_addr: Mutex::new(listen_addr),
            keep_open: AtomicBool::new(keep_open),
            web_lease: AtomicBool::new(false),
            listening: AtomicBool::new(keep_open),
            notify: tokio::sync::Notify::new(),
        }
    }

    pub fn listen_addr(&self) -> std::net::SocketAddr {
        *self
            .listen_addr
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub fn set_listen_addr(&self, addr: std::net::SocketAddr) {
        *self
            .listen_addr
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = addr;
    }

    pub fn keep_open(&self) -> bool {
        self.keep_open.load(Ordering::Acquire)
    }

    pub fn web_lease(&self) -> bool {
        self.web_lease.load(Ordering::Acquire)
    }

    pub fn should_listen(&self) -> bool {
        self.keep_open() || self.web_lease()
    }

    pub fn is_listening(&self) -> bool {
        self.listening.load(Ordering::Acquire)
    }

    pub fn set_keep_open(&self, keep: bool) {
        let prev = self.keep_open.swap(keep, Ordering::AcqRel);
        if prev != keep {
            if keep {
                // Explicit permanent open cancels any temporary lease flag.
                self.web_lease.store(false, Ordering::Release);
            }
            self.notify.notify_waiters();
        }
    }

    pub fn set_web_lease(&self, open: bool) {
        let prev = self.web_lease.swap(open, Ordering::AcqRel);
        if prev != open {
            self.notify.notify_waiters();
        }
    }

    /// Runs the TCP API server dynamically, starting or stopping the listener
    /// whenever `keep_open` or `web_lease` changes.
    pub async fn serve_tcp_controlled(
        self: Arc<Self>,
        app: Router,
        mut initial_listener: Option<tokio::net::TcpListener>,
        stop: CancellationToken,
    ) -> std::io::Result<()> {
        loop {
            if stop.is_cancelled() {
                self.listening.store(false, Ordering::Release);
                return Ok(());
            }

            if !self.should_listen() {
                if let Some(l) = initial_listener.take() {
                    drop(l);
                }
                self.listening.store(false, Ordering::Release);
                tokio::select! {
                    _ = stop.cancelled() => return Ok(()),
                    _ = self.notify.notified() => {}
                }
                continue;
            }

            let listener = match initial_listener.take() {
                Some(l) => l,
                None => {
                    let addr = self.listen_addr();
                    match bind_tcp_listener(addr) {
                        Ok(l) => l,
                        Err(error) => {
                            tracing::warn!(%error, %addr, "api: failed to re-bind tcp listener");
                            self.listening.store(false, Ordering::Release);
                            tokio::select! {
                                _ = stop.cancelled() => return Ok(()),
                                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                            }
                            continue;
                        }
                    }
                }
            };

            if let Ok(local) = listener.local_addr() {
                self.set_listen_addr(local);
            }
            self.listening.store(true, Ordering::Release);
            tracing::info!(address = %self.listen_addr(), "api: tcp listener active");

            let close_trigger = CancellationToken::new();
            let watcher_ctrl = Arc::clone(&self);
            let watcher_stop = stop.clone();
            let watcher_trigger = close_trigger.clone();
            let watcher = tokio::spawn(async move {
                loop {
                    if watcher_stop.is_cancelled() {
                        watcher_trigger.cancel();
                        break;
                    }
                    if !watcher_ctrl.should_listen() {
                        // Allow the HTTP response that disabled the port over TCP to flush.
                        tokio::time::sleep(Duration::from_millis(80)).await;
                        if !watcher_ctrl.should_listen() || watcher_stop.is_cancelled() {
                            watcher_trigger.cancel();
                            break;
                        }
                    }
                    tokio::select! {
                        _ = watcher_stop.cancelled() => {
                            watcher_trigger.cancel();
                            break;
                        }
                        _ = watcher_ctrl.notify.notified() => {}
                    }
                }
            });

            let drain_trigger = close_trigger.clone();
            let serve_fut = std::future::IntoFuture::into_future(
                axum::serve(
                    listener,
                    app.clone()
                        .into_make_service_with_connect_info::<std::net::SocketAddr>(),
                )
                .with_graceful_shutdown(close_trigger.cancelled_owned()),
            );

            tokio::pin!(serve_fut);
            let result = tokio::select! {
                res = &mut serve_fut => res,
                _ = async {
                    drain_trigger.cancelled().await;
                    tokio::time::sleep(Duration::from_millis(250)).await;
                } => Ok(()),
            };
            watcher.abort();
            self.listening.store(false, Ordering::Release);

            if stop.is_cancelled() {
                return result;
            }
            tracing::info!("api: tcp listener closed (Unix socket IPC stays active)");
        }
    }
}

/// Binds a non-blocking TCP listener using socket2 options matching `Service`.
pub fn bind_tcp_listener(addr: std::net::SocketAddr) -> std::io::Result<tokio::net::TcpListener> {
    let domain = match addr {
        std::net::SocketAddr::V4(_) => socket2::Domain::IPV4,
        std::net::SocketAddr::V6(_) => socket2::Domain::IPV6,
    };
    let socket = socket2::Socket::new(domain, socket2::Type::STREAM, None)?;
    socket.set_nonblocking(true)?;
    let _ = socket.set_reuse_address(true);
    #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
    let _ = socket.set_reuse_port(true);
    if addr.is_ipv6() {
        let _ = socket.set_only_v6(true);
    }
    let sock_addr = socket2::SockAddr::from(addr);
    socket.bind(&sock_addr)?;
    socket.listen(1024)?;
    let std_listener: std::net::TcpListener = socket.into();
    tokio::net::TcpListener::from_std(std_listener)
}

#[cfg(unix)]
struct UnixListenerAdapter {
    inner: tokio::net::UnixListener,
    local_path: PathBuf,
}

#[cfg(unix)]
impl axum::serve::Listener for UnixListenerAdapter {
    type Io = tokio::net::UnixStream;
    type Addr = String;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match self.inner.accept().await {
                Ok((stream, _addr)) => {
                    let peer_str = match stream.peer_cred() {
                        Ok(cred) => {
                            let pid = cred.pid().unwrap_or(0);
                            format!("unix:uid={}:pid={pid}", cred.uid())
                        }
                        Err(_) => "unix:uid=0:pid=0".to_string(),
                    };
                    return (stream, peer_str);
                }
                Err(error) => {
                    tracing::warn!(%error, "api: unix socket accept error");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    fn local_addr(&self) -> tokio::io::Result<Self::Addr> {
        Ok(format!("unix:{}", self.local_path.display()))
    }
}

#[cfg(unix)]
#[derive(Clone, Debug)]
struct UnixPeerAddr(String);

#[cfg(unix)]
impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, UnixListenerAdapter>>
    for UnixPeerAddr
{
    fn connect_info(stream: axum::serve::IncomingStream<'_, UnixListenerAdapter>) -> Self {
        UnixPeerAddr(stream.remote_addr().clone())
    }
}

#[cfg(unix)]
async fn attach_unix_peer_addr(mut request: AxumRequest, next: Next) -> AxumResponse {
    if let Some(ConnectInfo(peer)) = request
        .extensions()
        .get::<ConnectInfo<UnixPeerAddr>>()
        .cloned()
    {
        request.extensions_mut().insert(pm_api::PeerAddress(peer.0));
    }
    next.run(request).await
}

/// Serves the exact same API router over a local Unix Domain Socket (`SO_PEERCRED` auth).
#[cfg(unix)]
pub async fn serve_unix_socket(
    app: Router,
    socket_path: PathBuf,
    stop: CancellationToken,
) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    // Local UIDs authenticate via kernel SO_PEERCRED inside ApiAuthenticator, so
    // desktop GUIs running as the logged-in user need connect access when core runs as root.
    if let Err(error) = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666)) {
        tracing::warn!(%error, path = %socket_path.display(), "api: failed to set unix socket permissions");
    }
    tracing::info!(path = %socket_path.display(), "api: unix socket listening");

    let unix_app = app.layer(middleware::from_fn(attach_unix_peer_addr));
    let shutdown = stop.clone();
    let serve_fut = std::future::IntoFuture::into_future(
        axum::serve(
            UnixListenerAdapter {
                inner: listener,
                local_path: socket_path.clone(),
            },
            unix_app.into_make_service_with_connect_info::<UnixPeerAddr>(),
        )
        .with_graceful_shutdown(stop.cancelled_owned()),
    );

    tokio::pin!(serve_fut);
    let result = tokio::select! {
        res = &mut serve_fut => res,
        _ = async {
            shutdown.cancelled().await;
            tokio::time::sleep(Duration::from_millis(200)).await;
        } => Ok(()),
    };
    let _ = std::fs::remove_file(&socket_path);
    result
}

/// Legacy signature preserved for external callers/tests.
pub async fn serve(
    api: Arc<Api>,
    listener: tokio::net::TcpListener,
    config: Arc<Config>,
    data_dir: impl AsRef<Path>,
    bin_dir: impl AsRef<Path>,
) -> std::io::Result<()> {
    let app = build_router(api, config, data_dir, bin_dir);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
}

/// Converts an axum request into `pm_api::Request` and executes it on `Api`.
async fn handle_axum(
    State(api): State<Arc<Api>>,
    request: axum::extract::Request,
) -> AxumResponse {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, 20_000_000)
        .await
        .unwrap_or_default();

    let mut api_request = pm_api::Request::new(parts.method.as_str(), parts.uri.path());
    api_request.query_string = parts.uri.query().unwrap_or("").to_string();
    api_request.query = pm_api::parse_query(&api_request.query_string);
    api_request.host = parts
        .headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    api_request.remote_addr = parts
        .extensions
        .get::<pm_api::PeerAddress>()
        .map(|info| info.0.clone())
        .or_else(|| {
            parts
                .extensions
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|info| info.0.to_string())
        })
        .unwrap_or_default();
    api_request.header = parts.headers;
    api_request.body = body.to_vec();

    let response = match tokio::task::spawn_blocking(move || api.handle(api_request)).await {
        Ok(response) => response,
        Err(_) => pm_api::Response::error(500, "Internal server error."),
    };
    response.into_axum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_pilot_paths() {
        assert_eq!(
            clean_pilot_path("").unwrap().to_str(),
            Some("index.html")
        );
        assert_eq!(
            clean_pilot_path("app.js").unwrap().to_str(),
            Some("app.js")
        );
        assert_eq!(
            clean_pilot_path("assets/app.js").unwrap().to_str(),
            Some("assets/app.js")
        );
        assert_eq!(
            clean_pilot_path("/app.js"),
            None,
            "absolute paths must be rejected"
        );
        assert_eq!(clean_pilot_path(".."), None);
        assert_eq!(clean_pilot_path("a/../../secret"), None);
        assert_eq!(clean_pilot_path("a\\..\\secret"), None);
        assert_eq!(clean_pilot_path("C:/secret"), None);
    }

    #[test]
    fn resolves_only_files_inside_the_pilot_dir() {
        let root = std::env::temp_dir().join(format!("pm-core-pilot-test-{}", std::process::id()));
        let pilot = root.join(PILOT_ASSET_DIR);
        let nested = pilot.join("assets");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(pilot.join("index.html"), b"index").unwrap();
        std::fs::write(nested.join("app.js"), b"app").unwrap();
        std::fs::write(root.join("secret.txt"), b"secret").unwrap();

        let (index, _) = resolve_pilot_file(&pilot, "").unwrap();
        assert_eq!(index, std::fs::canonicalize(pilot.join("index.html")).unwrap());

        let (asset, _) = resolve_pilot_file(&pilot, "assets/app.js").unwrap();
        assert_eq!(asset, std::fs::canonicalize(nested.join("app.js")).unwrap());

        assert!(resolve_pilot_file(&pilot, "../secret.txt").is_none());
        assert!(resolve_pilot_file(&pilot, "missing.js").is_none());

        std::fs::remove_dir_all(&root).ok();
    }
}
