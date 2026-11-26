//! Reading and writing profiles in the `core` database.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use pm_database::{Interface, Key, Meta, Query, Record};

use crate::profile::Profile;
use crate::{make_profile_key, make_scoped_id, PROFILES_DB_PATH, SOURCE_LOCAL};

const META_CACHE_TTL: Duration = Duration::from_secs(5);

struct CachedMeta {
    meta: crate::meta::ProfilesMetadata,
    loaded_at: Instant,
}

/// Errors of the profile store.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("profile: {0}")]
    Database(#[from] pm_database::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("dsd: {0}")]
    Dsd(#[from] pm_dsd::Error),
    #[error("profile is invalid: {0}")]
    InvalidProfile(String),
}

/// Result type of the profile store.
pub type Result<T> = std::result::Result<T, Error>;

/// Stores profiles in the `core` database under `profiles/<source>/<id>`.
pub struct ProfileStore {
    db: Interface,
    meta_cache: Arc<RwLock<Option<CachedMeta>>>,
}

impl ProfileStore {
    /// Creates a store for the given database interface.
    ///
    /// The interface should have local and internal permissions.
    pub fn new(db: Interface) -> ProfileStore {
        ProfileStore {
            db,
            meta_cache: Arc::new(RwLock::new(None)),
        }
    }

    fn get_meta(&self) -> crate::meta::ProfilesMetadata {
        {
            if let Ok(guard) = self.meta_cache.read() {
                if let Some(ref cached) = *guard {
                    if cached.loaded_at.elapsed() < META_CACHE_TTL {
                        return cached.meta.clone();
                    }
                }
            }
        }
        let mut guard = self.meta_cache.write().unwrap_or_else(|p| p.into_inner());
        if let Some(ref cached) = *guard {
            if cached.loaded_at.elapsed() < META_CACHE_TTL {
                return cached.meta.clone();
            }
        }
        let meta = crate::meta::ProfilesMetadata::load(&self.db);
        *guard = Some(CachedMeta {
            meta: meta.clone(),
            loaded_at: Instant::now(),
        });
        meta
    }

    fn update_meta<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut crate::meta::ProfilesMetadata) -> R,
    {
        let mut guard = self.meta_cache.write().unwrap_or_else(|p| p.into_inner());
        let mut meta = match guard.take() {
            Some(cached) if cached.loaded_at.elapsed() < META_CACHE_TTL => cached.meta,
            _ => crate::meta::ProfilesMetadata::load(&self.db),
        };
        let result = f(&mut meta);
        if let Err(error) = meta.save(&self.db) {
            tracing::warn!(%error, "failed to save profile metadata");
        }
        *guard = Some(CachedMeta {
            meta,
            loaded_at: Instant::now(),
        });
        result
    }

    /// Reads a profile by its scoped ID (`source/id`).
    pub fn get(&self, scoped_id: &str) -> Result<Profile> {
        // Most callers pass a bare local ID; Go's getProfile() always builds
        // a scoped ID first. Accept both forms.
        let scoped_id = if scoped_id.contains('/') {
            scoped_id.to_string()
        } else {
            make_scoped_id(SOURCE_LOCAL, scoped_id)
        };
        let record = self.db.get(&format!("{PROFILES_DB_PATH}{scoped_id}"))?;
        Self::decode(&record)
    }

    /// Reads all profiles of a source.
    pub fn all(&self, source: &str) -> Result<Vec<Profile>> {
        let query = Query::new(&format!("{PROFILES_DB_PATH}{source}/"));
        let records = self.db.query(&query)?;
        let mut profiles = Vec::with_capacity(records.len());
        for record in &records {
            match Self::decode(record) {
                Ok(profile) => profiles.push(profile),
                Err(error) => {
                    tracing::warn!(key = %record.key, %error, "skipping invalid profile record");
                }
            }
        }
        Ok(profiles)
    }

    /// Rewrites profiles that still contain legacy wrongly-typed options.
    fn rewrite_local_configs(
        &self,
        mut transform: impl FnMut(serde_json::Map<String, serde_json::Value>) -> serde_json::Map<String, serde_json::Value>,
    ) -> Result<Vec<String>> {
        let records = self.db.query(&Query::new(PROFILES_DB_PATH))?;
        let mut updated_ids = Vec::new();
        for record in records {
            let mut profile: Profile = match serde_json::from_slice(&record.data) {
                Ok(profile) => profile,
                Err(_) => continue,
            };
            // Only rewrite local profiles.
            if profile.source != crate::SOURCE_LOCAL {
                continue;
            }
            if profile.id.is_empty() {
                let key_str = &record.key.key;
                if let Some(id) = key_str.strip_prefix(&format!("profiles/{}/", profile.source)) {
                    profile.id = id.to_string();
                } else if let Some(last_slash) = key_str.rfind('/') {
                    profile.id = key_str[last_slash + 1..].to_string();
                }
            }
            if profile.id.is_empty() {
                continue;
            }
            let updated = transform(profile.config.clone());
            if updated != profile.config {
                profile.config = updated;
                self.db.put(encode_profile(&profile)?)?;
                updated_ids.push(profile.id);
            }
        }
        Ok(updated_ids)
    }

    /// Rewrites local profiles that still carry legacy `usagePolicy` values.
    ///
    /// Returns the number of migrated records.
    pub fn migrate_legacy_configs(&self) -> Result<usize> {
        self.rewrite_local_configs(crate::profile::sanitize_legacy_config)
            .map(|ids| ids.len())
    }

    /// Removes per-app `spn/use` and `splittun/use` overrides from all local
    /// profiles so they inherit the global routing defaults cleanly.
    ///
    /// Returns the local IDs of profiles whose config changed.
    pub fn clear_routing_overrides(&self) -> Result<Vec<String>> {
        self.rewrite_local_configs(crate::profile::clear_routing_overrides_map)
    }

    /// Saves a profile.
    pub fn save(&self, profile: &Profile) -> Result<()> {
        if profile.id.is_empty() {
            return Err(Error::InvalidProfile("profile has no ID".to_string()));
        }
        if profile.source.is_empty() {
            return Err(Error::InvalidProfile(format!(
                "profile {} does not specify a source",
                profile.id
            )));
        }
        // Write the profile first; only mark it seen once the write succeeded,
        // so a failed put cannot leave phantom metadata behind.
        let mut clean = profile.clone();
        clean.config = crate::profile::sanitize_legacy_config(clean.config);
        self.db.put(self.encode(&clean)?)?;
        self.update_meta(|meta| {
            meta.mark_seen(&profile.id);
        });
        Ok(())
    }

    /// Deletes a profile.
    pub fn delete(&self, profile: &Profile) -> Result<()> {
        // Delete the record first; a failed delete must not mark a still
        // existing profile as deleted.
        self.db
            .delete(&make_profile_key(&profile.source, &profile.id))?;
        self.update_meta(|meta| {
            meta.mark_deleted(&profile.id);
        });
        Ok(())
    }

    /// Checks if a profile ID is marked as deleted.
    pub fn is_deleted(&self, id: &str) -> bool {
        let meta = self.get_meta();
        meta.is_deleted(id)
    }

    fn encode(&self, profile: &Profile) -> Result<Record> {
        encode_profile(profile)
    }

    fn decode(record: &Record) -> Result<Profile> {
        let mut profile: Profile = serde_json::from_slice(&record.data)?;
        profile.config = crate::profile::sanitize_legacy_config(profile.config);
        if profile.id.is_empty() {
            let key_str = &record.key.key;
            if let Some(id) = key_str.strip_prefix(&format!("profiles/{}/", profile.source)) {
                profile.id = id.to_string();
            } else if let Some(last_slash) = key_str.rfind('/') {
                profile.id = key_str[last_slash + 1..].to_string();
            }
        }
        Ok(profile)
    }
}

/// Encodes a profile as a database record, like Go `Profile.Encode`.
pub fn encode_profile(profile: &Profile) -> Result<Record> {
    Ok(Record {
        key: Key::parse(&profile.key()),
        meta: Meta {
            created: profile.created,
            modified: crate::now(),
            ..Meta::default()
        },
        format: pm_dsd::format::JSON,
        data: pm_dsd::gojson::to_vec(profile)?,
    })
}
