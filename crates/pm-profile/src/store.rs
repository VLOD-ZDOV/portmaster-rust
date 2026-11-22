//! Reading and writing profiles in the `core` database.

use pm_database::{Interface, Key, Meta, Query, Record};

use crate::profile::Profile;
use crate::{make_profile_key, make_scoped_id, PROFILES_DB_PATH, SOURCE_LOCAL};

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
}

impl ProfileStore {
    /// Creates a store for the given database interface.
    ///
    /// The interface should have local and internal permissions.
    pub fn new(db: Interface) -> ProfileStore {
        ProfileStore { db }
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
        let mut meta = crate::meta::ProfilesMetadata::load(&self.db);
        meta.mark_seen(&profile.id);
        let _ = meta.save(&self.db);
        self.db.put(self.encode(profile)?)?;
        Ok(())
    }

    /// Deletes a profile.
    pub fn delete(&self, profile: &Profile) -> Result<()> {
        let mut meta = crate::meta::ProfilesMetadata::load(&self.db);
        meta.mark_deleted(&profile.id);
        let _ = meta.save(&self.db);
        self.db
            .delete(&make_profile_key(&profile.source, &profile.id))?;
        Ok(())
    }

    /// Checks if a profile ID is marked as deleted.
    pub fn is_deleted(&self, id: &str) -> bool {
        let meta = crate::meta::ProfilesMetadata::load(&self.db);
        meta.is_deleted(id)
    }

    fn encode(&self, profile: &Profile) -> Result<Record> {
        encode_profile(profile)
    }

    fn decode(record: &Record) -> Result<Profile> {
        let mut profile: Profile = serde_json::from_slice(&record.data)?;
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
