//! Port of Go `updateBandwidth` and `packet.BandwidthUpdate`.

use std::sync::TryLockError;

use super::handler::HandlerCore;

/// How the byte counts of an update are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthMethod {
    /// The counts are the totals since the connection started.
    Absolute,
    /// The counts are added to the totals.
    Additive,
}

/// An update to the seen bandwidth of a connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandwidthUpdate {
    pub conn_id: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub method: BandwidthMethod,
}

impl HandlerCore {
    fn apply_bandwidth_update(&self, update: &BandwidthUpdate) -> Option<(bool, String, String, String, u64, u64)> {
        if update.bytes_received == 0 && update.bytes_sent == 0 {
            return None;
        }
        let shared = self.network.get_connection(&update.conn_id)?;
        let mut conn = match shared.try_lock() {
            Ok(conn) => conn,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return None,
        };

        let (bytes_in, bytes_out) = match update.method {
            BandwidthMethod::Absolute => {
                // Go subtracts unsigned and would wrap; never go below zero.
                let deltas = (
                    update.bytes_received.saturating_sub(conn.bytes_received),
                    update.bytes_sent.saturating_sub(conn.bytes_sent),
                );
                conn.bytes_received = update.bytes_received;
                conn.bytes_sent = update.bytes_sent;
                deltas
            }
            BandwidthMethod::Additive => {
                conn.bytes_received = conn.bytes_received.saturating_add(update.bytes_received);
                conn.bytes_sent = conn.bytes_sent.saturating_add(update.bytes_sent);
                (update.bytes_received, update.bytes_sent)
            }
        };

        if !conn.bandwidth_enabled {
            return None;
        }
        let process = conn.process.as_ref()?;
        let profile = format!(
            "{}/{}",
            conn.process_context.source, conn.process_context.profile
        );
        Some((
            conn.history_enabled,
            profile,
            process.key().to_string(),
            conn.id.clone(),
            bytes_in,
            bytes_out,
        ))
    }

    /// Applies a bandwidth update to the connection and forwards the delta to
    /// the sink. Connections that are locked are skipped, like in Go.
    pub fn update_bandwidth(&self, update: &BandwidthUpdate) {
        let Some(sink) = &self.bandwidth else {
            // Still update in-memory counters if no sink is attached.
            let _ = self.apply_bandwidth_update(update);
            return;
        };
        let Some((history_enabled, profile, process_key, conn_id, bytes_in, bytes_out)) =
            self.apply_bandwidth_update(update)
        else {
            return;
        };
        if let Err(error) = sink.update_bandwidth(
            history_enabled,
            &profile,
            &process_key,
            &conn_id,
            bytes_in,
            bytes_out,
        ) {
            tracing::error!("filter: failed to persist bandwidth data: {error}");
        }
    }

    /// Applies multiple bandwidth updates and flushes deltas to the sink in one batch.
    pub fn update_bandwidth_batch(&self, updates: &[BandwidthUpdate]) {
        if updates.is_empty() {
            return;
        }
        let Some(sink) = &self.bandwidth else {
            for u in updates {
                let _ = self.apply_bandwidth_update(u);
            }
            return;
        };
        let mut batched = Vec::with_capacity(updates.len());
        for u in updates {
            if let Some(entry) = self.apply_bandwidth_update(u) {
                batched.push(entry);
            }
        }
        if !batched.is_empty() {
            if let Err(error) = sink.update_bandwidth_batch(&batched) {
                tracing::error!("filter: failed to persist bandwidth batch: {error}");
            }
        }
    }
}
