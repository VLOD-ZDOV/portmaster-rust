//! Port of the listener handling of Go `module.go`: UDP listeners, failure
//! reporting and shutdown.

use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::conflict::{find_conflicting_process, ListeningProcessLookup};
use crate::config::{parse_listen_addresses, ListenAddressError};
use crate::handler::Nameserver;
use crate::matcher::NameserverIpMatcher;

/// How long a shutdown waits for requests that are still running.
pub const LISTENER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Where the notification of a software conflict points to.
pub const COMPATIBILITY_DOCS_URL: &str =
    "https://docs.safing.io/portmaster/install/status/software-compatibility";

const EVENT_ID_CONFLICTING_SERVICE: &str = "nameserver:conflicting-service";
const EVENT_ID_LISTENER_FAILED: &str = "nameserver:listener-failed";

/// A listener that could not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerFailure {
    /// The primary listener (the first address).
    pub primary: bool,
    pub addr: SocketAddr,
    pub error: String,
    /// The description of the software that holds the port, if found.
    pub conflict: Option<String>,
}

impl ListenerFailure {
    fn suffix(&self) -> &'static str {
        if self.primary {
            ""
        } else {
            "-secondary"
        }
    }

    pub fn is_conflict(&self) -> bool {
        self.conflict.is_some()
    }

    /// The ID of the notification.
    pub fn event_id(&self) -> String {
        let base = if self.is_conflict() {
            EVENT_ID_CONFLICTING_SERVICE
        } else {
            EVENT_ID_LISTENER_FAILED
        };
        format!("{base}{}", self.suffix())
    }

    pub fn title(&self) -> &'static str {
        if self.is_conflict() {
            "Conflicting DNS Software"
        } else {
            "Secure DNS Error"
        }
    }

    pub fn message(&self) -> String {
        match &self.conflict {
            Some(description) => format!(
                "Restart Portmaster after you have deactivated or properly configured the conflicting software: {description}"
            ),
            None => format!(
                "The internal DNS server failed. Restart Portmaster to try again. Error: {}",
                self.error
            ),
        }
    }
}

/// What happened to a listener.
#[derive(Debug, Clone)]
pub enum ListenerEvent {
    /// The listener runs; notifications of earlier failures of it (both
    /// event IDs, with the `-secondary` suffix for the second listener)
    /// should be removed.
    Started { primary: bool },
    Failed(ListenerFailure),
}

/// Receives listener events, e.g. to manage notifications.
pub type EventSink = Arc<dyn Fn(ListenerEvent) + Send + Sync>;

/// Optional parts of [`start`].
#[derive(Default)]
pub struct StartOptions {
    /// Finds the software that holds the port when binding fails.
    pub lookup: Option<Arc<dyn ListeningProcessLookup>>,
    pub on_event: Option<EventSink>,
}

struct Listener {
    addr: SocketAddr,
    task: JoinHandle<()>,
}

/// The running nameserver.
pub struct Running {
    listeners: Vec<Listener>,
    failures: Vec<ListenerFailure>,
    cancel: CancellationToken,
    matcher: NameserverIpMatcher,
}

impl Running {
    /// The addresses the listeners are bound to.
    pub fn local_addrs(&self) -> Vec<SocketAddr> {
        self.listeners.iter().map(|l| l.addr).collect()
    }

    /// The listeners that could not start.
    pub fn failures(&self) -> &[ListenerFailure] {
        &self.failures
    }

    /// The matcher for the firewall fast-track.
    pub fn ip_matcher(&self) -> NameserverIpMatcher {
        self.matcher.clone()
    }

    /// Stops the listeners and waits for running requests, up to
    /// [`LISTENER_SHUTDOWN_TIMEOUT`].
    pub async fn shutdown(mut self) {
        self.cancel.cancel();
        let mut listeners = std::mem::take(&mut self.listeners);
        // One deadline for all listeners; they drain their requests in parallel.
        let wait = async {
            for listener in &mut listeners {
                if let Err(err) = (&mut listener.task).await {
                    tracing::warn!("nameserver: failed to stop listener: {err}");
                }
            }
        };
        if tokio::time::timeout(LISTENER_SHUTDOWN_TIMEOUT + Duration::from_secs(1), wait)
            .await
            .is_err()
        {
            for listener in &listeners {
                listener.task.abort();
            }
        }
    }
}

impl Drop for Running {
    /// Dropping the handle stops the listeners.
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

async fn serve(
    socket: Arc<UdpSocket>,
    ns: Arc<Nameserver>,
    cancel: CancellationToken,
    addr: SocketAddr,
    primary: bool,
    on_event: Option<EventSink>,
) {
    let mut buf = vec![0u8; 65_535];
    let mut requests = JoinSet::new();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            // Reap finished requests.
            Some(_) = requests.join_next(), if !requests.is_empty() => {}
            received = socket.recv_from(&mut buf) => {
                let (n, remote) = match received {
                    Ok(received) => received,
                    Err(err) if matches!(
                        err.kind(),
                        ErrorKind::WouldBlock
                            | ErrorKind::Interrupted
                            | ErrorKind::ConnectionReset
                            | ErrorKind::ConnectionRefused
                    ) => {
                        // Errors of single datagrams (e.g. ICMP) do not end the listener.
                        tracing::debug!("nameserver: failed to read from {addr}: {err}");
                        continue;
                    }
                    Err(err) => {
                        // Like the server library, stop on other errors instead of spinning.
                        tracing::warn!("nameserver: failed to serve on {addr}: {err}");
                        if let Some(sink) = &on_event {
                            sink(ListenerEvent::Failed(ListenerFailure {
                                primary,
                                addr,
                                error: err.to_string(),
                                conflict: None,
                            }));
                        }
                        break;
                    }
                };
                let packet = buf[..n].to_vec();
                let ns = Arc::clone(&ns);
                let socket = Arc::clone(&socket);
                let request_cancel = cancel.clone();
                requests.spawn(async move {
                    let Some(reply) = ns
                        .handle_packet_with_cancel(&packet, remote, &request_cancel)
                        .await
                    else {
                        return;
                    };
                    if let Err(err) = socket.send_to(&reply, remote).await {
                        tracing::warn!("nameserver: failed to send response: {err}");
                    }
                });
            }
        }
    }

    // Wait for requests that are still running; abandon them afterwards.
    let wait = async { while requests.join_next().await.is_some() {} };
    if tokio::time::timeout(LISTENER_SHUTDOWN_TIMEOUT, wait).await.is_err() {
        tracing::warn!("nameserver: abandoning requests on {addr}");
        requests.abort_all();
    }
}

fn is_addr_in_use(err: &std::io::Error) -> bool {
    err.kind() == ErrorKind::AddrInUse || err.raw_os_error() == Some(10048)
}

/// Starts the listeners for the configured address, like Go `start`.
///
/// Listeners that cannot bind do not fail the start; they are reported in
/// [`Running::failures`] and through the event sink.
pub async fn start(
    ns: Arc<Nameserver>,
    listen_address: &str,
    options: StartOptions,
) -> Result<Running, ListenAddressError> {
    let addresses = parse_listen_addresses(listen_address, ns.env().ipv6_enabled())?;

    let mut targets = vec![(addresses.ip1, true)];
    if let Some(ip2) = addresses.ip2 {
        targets.push((ip2, false));
    }

    let cancel = CancellationToken::new();
    let mut listeners = Vec::new();
    let mut failures = Vec::new();
    for (ip, primary) in targets {
        let addr = SocketAddr::new(ip, addresses.port);
        tracing::info!("nameserver: starting to listen on {addr}");
        match UdpSocket::bind(addr).await {
            Ok(socket) => {
                let local = socket.local_addr().unwrap_or(addr);
                if let Some(sink) = &options.on_event {
                    sink(ListenerEvent::Started { primary });
                }
                let task = tokio::spawn(serve(
                    Arc::new(socket),
                    Arc::clone(&ns),
                    cancel.clone(),
                    local,
                    primary,
                    options.on_event.clone(),
                ));
                listeners.push(Listener { addr: local, task });
            }
            Err(err) => {
                // Check if binding to an unspecified address failed because the port
                // is already in use (e.g. Windows SharedAccess / Internet Connection Sharing).
                if ip.is_unspecified() && is_addr_in_use(&err) {
                    tracing::warn!(
                        "nameserver: address {addr} is in use (WSAEADDRINUSE / 10048, likely Windows ICS / SharedAccess); attempting fallback bind to loopback"
                    );

                    let fallback_ips: &[IpAddr] = if ip.is_ipv6() {
                        &[IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)]
                    } else {
                        &[
                            IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                            IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 17)),
                        ]
                    };

                    let mut fallback_success = false;
                    for &fallback_ip in fallback_ips {
                        let fallback_addr = SocketAddr::new(fallback_ip, addresses.port);
                        tracing::info!("nameserver: trying fallback listen on {fallback_addr}");
                        match UdpSocket::bind(fallback_addr).await {
                            Ok(socket) => {
                                let local = socket.local_addr().unwrap_or(fallback_addr);
                                tracing::info!(
                                    "nameserver: fallback listener successfully bound to {local}"
                                );
                                if let Some(sink) = &options.on_event {
                                    sink(ListenerEvent::Started { primary });
                                }
                                let task = tokio::spawn(serve(
                                    Arc::new(socket),
                                    Arc::clone(&ns),
                                    cancel.clone(),
                                    local,
                                    primary,
                                    options.on_event.clone(),
                                ));
                                listeners.push(Listener { addr: local, task });
                                fallback_success = true;
                                break;
                            }
                            Err(fallback_err) => {
                                tracing::warn!(
                                    "nameserver: fallback bind to {fallback_addr} failed: {fallback_err}"
                                );
                            }
                        }
                    }

                    if fallback_success {
                        continue;
                    }
                }

                // Probably some other application listens on the same port.
                tracing::warn!("nameserver: failed to start listening on {addr}: {err}");
                // The lookup reads socket tables and the proc file system.
                let conflict = match options.lookup.clone() {
                    Some(lookup) => {
                        let port = addresses.port;
                        tokio::task::spawn_blocking(move || {
                            find_conflicting_process(lookup.as_ref(), ip, port)
                        })
                        .await
                        .ok()
                        .flatten()
                        .map(|process| process.description())
                    }
                    None => None,
                };
                let failure = ListenerFailure {
                    primary,
                    addr,
                    error: err.to_string(),
                    conflict,
                };
                if let Some(sink) = &options.on_event {
                    sink(ListenerEvent::Failed(failure.clone()));
                }
                failures.push(failure);
            }
        }
    }

    // The matcher fast-tracks queries to our own addresses.
    let port = listeners
        .first()
        .map(|l| l.addr.port())
        .unwrap_or(addresses.port);
    let matcher = if addresses.ip2.is_none()
        && addresses.ip1.is_unspecified()
        && listeners.iter().any(|l| l.addr.ip().is_unspecified())
    {
        NameserverIpMatcher::new(Vec::new(), Some(Arc::clone(ns.env())), port)
    } else {
        let mut ips: Vec<IpAddr> = listeners.iter().map(|l| l.addr.ip()).collect();
        if ips.is_empty() {
            ips.push(addresses.ip1);
            if let Some(ip2) = addresses.ip2 {
                ips.push(ip2);
            }
        }
        NameserverIpMatcher::new(ips, None, port)
    };

    Ok(Running {
        listeners,
        failures,
        cancel,
        matcher,
    })
}
