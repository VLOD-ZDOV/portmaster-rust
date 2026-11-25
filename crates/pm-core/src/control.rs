//! Wiring for the Go `service/control` pause/resume module.
//!
//! The Rust core has no SPN service; `spn/enable` is read from the live config
//! and updated by the control module, so Xray/API/tray changes stay visible.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use pm_api::{Api, ApiError, PERMIT_ADMIN};
use pm_config::Config;
use pm_control::{Control, ControlEnv, NotificationSink, API_ENDPOINT_PAUSE, API_ENDPOINT_RESUME};
use pm_notifications::Notification;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

/// Environment adapter over the core's config, firewall and shutdown token.
pub struct CoreControlEnv {
    config: Arc<Config>,
    firewall: Arc<crate::firewall::Firewall>,
    shutting_down: CancellationToken,
}

impl CoreControlEnv {
    pub fn new(
        config: Arc<Config>,
        firewall: Arc<crate::firewall::Firewall>,
        shutting_down: CancellationToken,
    ) -> CoreControlEnv {
        CoreControlEnv {
            config,
            firewall,
            shutting_down,
        }
    }
}

#[async_trait]
impl ControlEnv for CoreControlEnv {
    fn is_shutting_down(&self) -> bool {
        self.shutting_down.is_cancelled()
    }

    fn spn_enabled(&self) -> bool {
        // Always read the live option: Xray start/API/tray changes it
        // asynchronously, and a cached AtomicBool would make "Pause VPN"
        // fail or resume the wrong module.
        self.config.get_as_bool("spn/enable", false)()
    }

    fn set_spn_enabled(&self, enabled: bool) {
        // The Xray module registers `spn/enable`; keep it in sync when present.
        if self.config.option("spn/enable").is_ok() {
            if let Err(error) = self
                .config
                .set_config_option("spn/enable", serde_json::Value::Bool(enabled))
            {
                tracing::warn!(%error, "control: failed to persist spn/enable");
            }
        }
    }

    fn spn_is_stopped(&self) -> bool {
        // No SPN service is running in the Rust core.
        true
    }

    async fn stop_interception(&self) -> pm_control::Result<()> {
        self.firewall.stop().await.map_err(|error| {
            pm_control::Error::Message(format!("failed to stop interception: {error}"))
        })
    }

    async fn start_interception(&self) -> pm_control::Result<()> {
        self.firewall.start().map_err(|error| {
            pm_control::Error::Message(format!("failed to start interception: {error}"))
        })
    }

    fn on_resume(&self) {
        // The resume path starts interception before enabling Xray, so the
        // firewall saw spn/enable=false. Flush stale flows now that both
        // switches are active.
        if self.spn_enabled() {
            if let Err(error) = self.firewall.delete_unmarked() {
                tracing::warn!(%error, "control: failed to flush unmarked conntrack on resume");
            }
        }
    }
}

/// Bridges the control module to the core's notification registry.
pub struct CoreNotificationSink(pub Arc<pm_notifications::Notifications>);

impl NotificationSink for CoreNotificationSink {
    fn notify(&self, notification: Notification) {
        self.0.notify(notification);
    }

    fn delete(&self, event_id: &str) {
        self.0.delete(event_id);
    }
}

#[derive(Debug, Default, Deserialize)]
struct PauseRequest {
    /// Duration in seconds.
    #[serde(default)]
    duration: i64,
    #[serde(default, rename = "onlySPN")]
    only_spn: bool,
}

/// Registers `control/pause` and `control/resume`.
pub fn register_api(api: &Api, control: Arc<Control<CoreControlEnv>>) -> Result<(), ApiError> {
    let pause_control = Arc::clone(&control);
    api.register_endpoint(
        pm_api::Endpoint::new(
            API_ENDPOINT_PAUSE,
            "Pause Portmaster",
            "Pause the Portmaster Core Service.",
        )
        .with_write(PERMIT_ADMIN)
        .with_write_method("POST")
        .with_parameter(pm_api::Parameter {
            method: "POST".to_string(),
            field: "duration".to_string(),
            description: "Specify the duration to pause the service in seconds.".to_string(),
            value: String::new(),
        })
        .with_parameter(pm_api::Parameter {
            method: "POST".to_string(),
            field: "onlySPN".to_string(),
            description: "Specify whether to pause only the SPN service.".to_string(),
            value: "false".to_string(),
        })
        .with_action(move |request| {
            let params: PauseRequest = request
                .body_json()
                .map_err(|error| ApiError::Message(format!("Bad Request: invalid input data: {error}")).with_status(400))?;
            if params.duration <= 0 {
                return Err(ApiError::Message("invalid pause duration".to_string()).with_status(400));
            }
            let control = Arc::clone(&pause_control);
            let outcome = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(control.pause(
                    Duration::from_secs(params.duration as u64),
                    params.only_spn,
                ))
            });
            match outcome {
                Ok(()) => Ok("Pause initiated".to_string()),
                Err(error) => Err(ApiError::Message(format!("Failed to pause: {error}"))
                    .with_status(500)),
            }
        }),
    )?;

    let resume_control = Arc::clone(&control);
    api.register_endpoint(
        pm_api::Endpoint::new(
            API_ENDPOINT_RESUME,
            "Resume Portmaster",
            "Resume the Portmaster Core Service.",
        )
        .with_write(PERMIT_ADMIN)
        .with_write_method("POST")
        .with_action(move |_| {
            let control = Arc::clone(&resume_control);
            let outcome = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(control.resume())
            });
            match outcome {
                Ok(()) => Ok("Resume initiated".to_string()),
                Err(error) => Err(ApiError::Message(format!("Failed to resume: {error}"))
                    .with_status(500)),
            }
        }),
    )?;

    Ok(())
}
