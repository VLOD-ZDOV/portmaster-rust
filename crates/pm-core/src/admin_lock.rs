//! Optional Admin Password / PIN Lock with 1-hour session caching and
//! owner UID/username trust filtering.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use pm_api::{Api, ApiError, Endpoint, Request, Response, PERMIT_ADMIN, PERMIT_USER};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

const DEFAULT_SESSION_TTL_SECS: u64 = 3600;
const LOCK_FILE_NAME: &str = "admin_lock.json";
pub const ADMIN_TOKEN_HEADER: &str = "x-portmaster-admin-token";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedLockConfig {
    enabled: bool,
    #[serde(default)]
    salt_hex: String,
    #[serde(default)]
    hash_hex: String,
    #[serde(default)]
    created_by: String,
    #[serde(default)]
    owner_uid: Option<u32>,
    #[serde(default)]
    auto_approve_owner: bool,
    #[serde(default = "default_ttl")]
    session_ttl_secs: u64,
    #[serde(default = "default_true")]
    allow_prompts_when_locked: bool,
    #[serde(default)]
    allow_xray_when_locked: bool,
}

fn default_ttl() -> u64 {
    DEFAULT_SESSION_TTL_SECS
}

fn default_true() -> bool {
    true
}

impl Default for PersistedLockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            salt_hex: String::new(),
            hash_hex: String::new(),
            created_by: String::new(),
            owner_uid: None,
            auto_approve_owner: false,
            session_ttl_secs: DEFAULT_SESSION_TTL_SECS,
            allow_prompts_when_locked: true,
            allow_xray_when_locked: false,
        }
    }
}

#[derive(Debug, Clone)]
struct UnlockedSession {
    expires_at: u64,
    peer_uid: Option<u32>,
}

struct LockState {
    config: PersistedLockConfig,
    sessions: HashMap<String, UnlockedSession>,
}

/// Optional administrator lock guarding mutating `PERMIT_ADMIN` operations
/// only after the user explicitly configures a password.
pub struct AdminLock {
    file_path: PathBuf,
    state: RwLock<LockState>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockRequest {
    password: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SetPasswordRequest {
    current_password: Option<String>,
    new_password: Option<String>,
    created_by: Option<String>,
    auto_approve_owner: Option<bool>,
    session_ttl_secs: Option<u64>,
    allow_prompts_when_locked: Option<bool>,
    allow_xray_when_locked: Option<bool>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn hash_password(salt_hex: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt_hex.as_bytes());
    hasher.update(b":pm-admin-v1:");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn random_hex(bytes_len: usize) -> String {
    let mut buf = vec![0u8; bytes_len];
    let _ = pm_rng::read(&mut buf);
    hex::encode(buf)
}

fn parse_peer_uid(peer: Option<&str>) -> Option<u32> {
    let peer = peer?;
    let rest = peer.strip_prefix("unix:uid=")?;
    let uid_str = rest.split(':').next()?;
    uid_str.parse::<u32>().ok()
}

fn resolve_unix_username(uid: u32) -> Option<String> {
    #[cfg(unix)]
    {
        if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
            let uid_s = uid.to_string();
            for line in passwd.lines() {
                let mut cols = line.split(':');
                let Some(name) = cols.next() else { continue };
                let _pw = cols.next();
                let Some(id) = cols.next() else { continue };
                if id == uid_s && !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        Some(format!("uid:{uid}"))
    }
    #[cfg(not(unix))]
    {
        let _ = uid;
        None
    }
}

impl AdminLock {
    pub fn load(data_dir: &Path) -> Arc<Self> {
        let file_path = data_dir.join(LOCK_FILE_NAME);
        let config = std::fs::read(&file_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<PersistedLockConfig>(&bytes).ok())
            .unwrap_or_default();
        Arc::new(Self {
            file_path,
            state: RwLock::new(LockState {
                config,
                sessions: HashMap::new(),
            }),
        })
    }

    fn persist(path: &Path, cfg: &PersistedLockConfig) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(cfg)
            .map_err(|e| std::io::Error::other(format!("serialize admin lock: {e}")))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn prune_expired(state: &mut LockState, now: u64) {
        state.sessions.retain(|_, sess| sess.expires_at > now);
    }

    fn has_valid_unlock(&self, token: Option<&str>, peer: Option<&str>) -> bool {
        let now = now_secs();
        let peer_uid = parse_peer_uid(peer);
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        if !state.config.enabled || state.config.hash_hex.is_empty() {
            return true;
        }
        if state.config.auto_approve_owner {
            if let (Some(owner), Some(caller)) = (state.config.owner_uid, peer_uid) {
                if owner == caller {
                    return true;
                }
            }
        }
        if let Some(tok) = token.map(str::trim).filter(|t| !t.is_empty()) {
            if let Some(sess) = state.sessions.get(tok) {
                if sess.expires_at > now {
                    return true;
                }
            }
        }
        // Check cached per-peer UID session so freshly reconnected WS or sibling
        // calls from the same user desktop stay unlocked for the 1-hour session.
        if let Some(uid) = peer_uid {
            if state
                .sessions
                .values()
                .any(|sess| sess.expires_at > now && sess.peer_uid == Some(uid))
            {
                return true;
            }
        }
        false
    }

    fn status_json(&self, token: Option<&str>, peer: Option<&str>) -> serde_json::Value {
        let now = now_secs();
        let peer_uid = parse_peer_uid(peer);
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        let password_set = state.config.enabled && !state.config.hash_hex.is_empty();
        let mut unlocked_until = 0u64;
        if password_set {
            if let Some(tok) = token.map(str::trim).filter(|t| !t.is_empty()) {
                if let Some(sess) = state.sessions.get(tok).filter(|s| s.expires_at > now) {
                    unlocked_until = unlocked_until.max(sess.expires_at);
                }
            }
            if let Some(uid) = peer_uid {
                for sess in state.sessions.values() {
                    if sess.expires_at > now && sess.peer_uid == Some(uid) {
                        unlocked_until = unlocked_until.max(sess.expires_at);
                    }
                }
            }
        }
        let locked = password_set && !self.has_valid_unlock(token, peer);
        json!({
            "enabled": password_set,
            "passwordSet": password_set,
            "locked": locked,
            "unlockedUntil": unlocked_until,
            "sessionTtlSecs": state.config.session_ttl_secs,
            "createdBy": state.config.created_by,
            "autoApproveOwner": state.config.auto_approve_owner,
            "allowPromptsWhenLocked": state.config.allow_prompts_when_locked,
            "allowXrayWhenLocked": state.config.allow_xray_when_locked,
        })
    }

    pub fn unlock(&self, password: &str, peer: Option<&str>) -> std::result::Result<(String, u64), &'static str> {
        let now = now_secs();
        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
        Self::prune_expired(&mut state, now);
        if !state.config.enabled || state.config.hash_hex.is_empty() {
            return Ok((String::new(), 0));
        }
        let candidate = hash_password(&state.config.salt_hex, password);
        if !constant_time_eq(candidate.as_bytes(), state.config.hash_hex.as_bytes()) {
            return Err("Неверный пароль администратора");
        }
        let token = random_hex(24);
        let ttl = state.config.session_ttl_secs.clamp(60, 86_400);
        let expires_at = now.saturating_add(ttl);
        state.sessions.insert(
            token.clone(),
            UnlockedSession {
                expires_at,
                peer_uid: parse_peer_uid(peer),
            },
        );
        Ok((token, expires_at))
    }

    pub fn lock_now(&self) {
        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
        state.sessions.clear();
    }

    fn is_http_allowed_while_locked(&self, request: &Request, endpoint: &Endpoint) -> bool {
        let req_perm = if request.read_method {
            endpoint.read
        } else {
            endpoint.write
        };
        if req_perm < PERMIT_ADMIN {
            return true;
        }
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        let p = endpoint.path.as_str();
        if matches!(p, "auth/status" | "auth/unlock" | "auth/lock") {
            return true;
        }
        if state.config.allow_xray_when_locked
            && matches!(p, "xray/connect" | "xray/connect-last" | "xray/disconnect" | "xray/toggle")
        {
            return true;
        }
        false
    }

    fn is_ws_key_allowed_while_locked(&self, key: &str) -> bool {
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        state.config.allow_prompts_when_locked && key.starts_with("notifications:")
    }

    pub fn install(self: &Arc<Self>, api: &Api) -> pm_api::Result<()> {
        let http_self = Arc::clone(self);
        api.set_lock_guard(Arc::new(move |request, endpoint| {
            let token = request
                .header(ADMIN_TOKEN_HEADER)
                .or_else(|| request.query_param("admin_token"));
            let peer = if request.remote_addr.is_empty() {
                None
            } else {
                Some(request.remote_addr.as_str())
            };
            if http_self.has_valid_unlock(token, peer)
                || http_self.is_http_allowed_while_locked(request, endpoint)
            {
                return Ok(());
            }
            let body = serde_json::to_vec(&json!({
                "locked": true,
                "error": "Требуется разблокировка паролем администратора",
            }))
            .unwrap_or_default();
            Err(Response::body(403, body, "application/json"))
        }));

        let ws_self = Arc::clone(self);
        api.set_ws_write_guard(Arc::new(move |token, peer, key| {
            if ws_self.has_valid_unlock(token, peer) || ws_self.is_ws_key_allowed_while_locked(key) {
                Ok(())
            } else {
                Err("locked: admin password required".to_string())
            }
        }));

        let status_lock = Arc::clone(self);
        api.register_endpoint(
            Endpoint::new(
                "auth/status",
                "Admin Lock Status",
                "Returns whether optional administrator lock is active and session status.",
            )
            .with_read(PERMIT_USER)
            .with_struct(move |request| {
                let token = request
                    .header(ADMIN_TOKEN_HEADER)
                    .or_else(|| request.query_param("admin_token"));
                let peer = (!request.remote_addr.is_empty()).then_some(request.remote_addr.as_str());
                Ok(status_lock.status_json(token, peer))
            }),
        )?;

        let unlock_lock = Arc::clone(self);
        api.register_endpoint(
            Endpoint::new(
                "auth/unlock",
                "Unlock Admin Session",
                "Verifies admin password and caches unlocked permissions for the configured session TTL (default 1 hour).",
            )
            .with_write(PERMIT_USER)
            .with_write_method("POST")
            .with_struct(move |request| {
                let payload: UnlockRequest = request.body_json()?;
                let peer = if request.remote_addr.is_empty() {
                None
            } else {
                Some(request.remote_addr.as_str())
            };
                match unlock_lock.unlock(&payload.password, peer) {
                    Ok((token, expires_at)) => {
                        let ttl = unlock_lock
                            .state
                            .read()
                            .unwrap_or_else(|e| e.into_inner())
                            .config
                            .session_ttl_secs;
                        Ok(json!({
                            "unlocked": true,
                            "token": token,
                            "expiresAt": expires_at,
                            "sessionTtlSecs": ttl,
                        }))
                    }
                    Err(msg) => Err(ApiError::WithStatus {
                        status: 403,
                        message: msg.to_string(),
                    }),
                }
            }),
        )?;

        let lock_ep = Arc::clone(self);
        api.register_endpoint(
            Endpoint::new(
                "auth/lock",
                "Lock Admin Session",
                "Clears all cached admin unlock sessions immediately.",
            )
            .with_write(PERMIT_USER)
            .with_write_method("POST")
            .with_struct(move |request| {
                lock_ep.lock_now();
                let peer = (!request.remote_addr.is_empty()).then_some(request.remote_addr.as_str());
                Ok(lock_ep.status_json(None, peer))
            }),
        )?;

        let set_pw = Arc::clone(self);
        api.register_endpoint(
            Endpoint::new(
                "auth/set-password",
                "Configure Admin Lock",
                "Sets, updates, or clears the optional administrator lock and its permissions.",
            )
            .with_write(PERMIT_ADMIN)
            .with_write_method("POST")
            .with_struct(move |request| {
                let payload: SetPasswordRequest = request.body_json()?;
                let token = request
                    .header(ADMIN_TOKEN_HEADER)
                    .or_else(|| request.query_param("admin_token"));
                let peer = if request.remote_addr.is_empty() {
                None
            } else {
                Some(request.remote_addr.as_str())
            };
                let peer_uid = parse_peer_uid(peer);

                let now = now_secs();
                let mut state = set_pw.state.write().unwrap_or_else(|e| e.into_inner());
                let currently_set = state.config.enabled && !state.config.hash_hex.is_empty();
                let already_unlocked = token
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .and_then(|t| state.sessions.get(t))
                    .is_some_and(|s| s.expires_at > now);

                if currently_set && !already_unlocked {
                    let cur = payload.current_password.as_deref().unwrap_or("");
                    let cand = hash_password(&state.config.salt_hex, cur);
                    if !constant_time_eq(cand.as_bytes(), state.config.hash_hex.as_bytes()) {
                        return Err(ApiError::WithStatus {
                            status: 403,
                            message: "Неверный текущий пароль администратора".to_string(),
                        });
                    }
                }

                if let Some(new_pw) = payload.new_password {
                    if new_pw.is_empty() {
                        state.config.enabled = false;
                        state.config.salt_hex.clear();
                        state.config.hash_hex.clear();
                        state.sessions.clear();
                    } else {
                        let salt = random_hex(16);
                        let hash = hash_password(&salt, &new_pw);
                        state.config.salt_hex = salt;
                        state.config.hash_hex = hash;
                        state.config.enabled = true;
                        state.config.owner_uid = peer_uid;
                        if state.config.created_by.is_empty() {
                            if let Some(uid) = peer_uid {
                                if let Some(name) = resolve_unix_username(uid) {
                                    state.config.created_by = name;
                                }
                            }
                        }
                    }
                }

                if let Some(by) = payload.created_by {
                    state.config.created_by = by.trim().to_string();
                } else if state.config.created_by.is_empty() {
                    if let Some(uid) = peer_uid {
                        if let Some(name) = resolve_unix_username(uid) {
                            state.config.created_by = name;
                        }
                    }
                }
                if let Some(auto_owner) = payload.auto_approve_owner {
                    state.config.auto_approve_owner = auto_owner;
                }
                if let Some(ttl) = payload.session_ttl_secs {
                    state.config.session_ttl_secs = ttl.clamp(60, 86_400);
                }
                if let Some(prompts) = payload.allow_prompts_when_locked {
                    state.config.allow_prompts_when_locked = prompts;
                }
                if let Some(xray_ok) = payload.allow_xray_when_locked {
                    state.config.allow_xray_when_locked = xray_ok;
                }

                Self::persist(&set_pw.file_path, &state.config).map_err(|e| {
                    ApiError::WithStatus {
                        status: 500,
                        message: format!("failed to persist admin lock config: {e}"),
                    }
                })?;

                let mut issued_token = String::new();
                let mut expires_at = 0u64;
                if state.config.enabled {
                    issued_token = random_hex(24);
                    expires_at = now.saturating_add(state.config.session_ttl_secs);
                    state.sessions.insert(
                        issued_token.clone(),
                        UnlockedSession {
                            expires_at,
                            peer_uid,
                        },
                    );
                }

                Ok(json!({
                    "enabled": state.config.enabled,
                    "passwordSet": state.config.enabled,
                    "locked": false,
                    "token": issued_token,
                    "expiresAt": expires_at,
                    "sessionTtlSecs": state.config.session_ttl_secs,
                    "createdBy": state.config.created_by,
                    "autoApproveOwner": state.config.auto_approve_owner,
                    "allowPromptsWhenLocked": state.config.allow_prompts_when_locked,
                    "allowXrayWhenLocked": state.config.allow_xray_when_locked,
                }))
            }),
        )?;

        Ok(())
    }
}
