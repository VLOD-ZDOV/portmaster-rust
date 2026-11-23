//! Metadata tracking profile states (e.g. deleted vs seen), port of Go `service/profile/meta.go`.

use std::collections::HashMap;
use pm_database::{Interface, Key, Meta, Record};
use serde::{Deserialize, Serialize};

pub const PROFILES_METADATA_KEY: &str = "core:profile-states";
pub const META_STATE_SEEN: &str = "seen";
pub const META_STATE_DELETED: &str = "deleted";

/// Deleted markers are kept for 30 days, like Go `removeDeletedEntriesAfter`.
const REMOVE_DELETED_ENTRIES_AFTER: i64 = 30 * 24 * 60 * 60;

/// Describes the life-cycle state of a profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaState {
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "At", default)]
    pub at: i64,
}

/// Metadata table keeping track of deleted profiles so they do not resurrect.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilesMetadata {
    #[serde(rename = "States", default)]
    pub states: HashMap<String, MetaState>,
}

impl ProfilesMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads the profile metadata record from database.
    pub fn load(db: &Interface) -> Self {
        match db.get(PROFILES_METADATA_KEY) {
            Ok(record) => match serde_json::from_slice(&record.data) {
                Ok(meta) => meta,
                Err(error) => {
                    // A malformed record must not pass silently: losing the
                    // deleted markers resurrects removed profiles.
                    tracing::error!(
                        %error,
                        "profile metadata record is malformed, starting with empty state; \
                         previously deleted profiles may reappear"
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Removes expired deleted markers, like Go `ProfilesMetadata.Clean`.
    pub fn clean(&mut self) {
        let now = crate::now();
        self.states.retain(|_, state| {
            state.state != META_STATE_DELETED || now - state.at <= REMOVE_DELETED_ENTRIES_AFTER
        });
    }

    /// Saves the metadata record back to the database.
    pub fn save(&self, db: &Interface) -> Result<(), pm_database::Error> {
        let mut cleaned = self.clone();
        cleaned.clean();
        let data = serde_json::to_vec(&cleaned)
            .map_err(|_| pm_database::Error::RecordMalformed)?;
        // Preserve the original creation timestamp across saves.
        let created = db
            .get_meta(PROFILES_METADATA_KEY)
            .map(|meta| meta.created)
            .unwrap_or(0);
        db.put(Record {
            key: Key::parse(PROFILES_METADATA_KEY),
            meta: Meta {
                created,
                modified: crate::now(),
                ..Meta::default()
            },
            format: pm_dsd::format::JSON,
            data,
        })
    }

    /// Marks a profile ID as deleted.
    pub fn mark_deleted(&mut self, id: &str) {
        self.states.insert(
            id.to_string(),
            MetaState {
                state: META_STATE_DELETED.to_string(),
                at: crate::now(),
            },
        );
    }

    /// Marks a profile ID as seen if not already recorded.
    pub fn mark_seen(&mut self, id: &str) {
        self.states.entry(id.to_string()).or_insert_with(|| MetaState {
            state: META_STATE_SEEN.to_string(),
            at: crate::now(),
        });
    }

    /// Returns whether the profile ID has been marked deleted.
    pub fn is_deleted(&self, id: &str) -> bool {
        self.states
            .get(id)
            .is_some_and(|s| s.state == META_STATE_DELETED)
    }
}
