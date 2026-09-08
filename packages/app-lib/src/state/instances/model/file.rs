use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceFile {
    pub id: String,
    pub instance_id: String,
    pub relative_path: String,
    pub file_name: String,
    pub enabled: bool,
    pub sha1: String,
    pub size: u64,
    pub missing: bool,
    pub added_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    /// JSON-encoded `LocalModMetadata` extracted from the JAR's embedded
    /// mod metadata file (fabric.mod.json, quilt.mod.json, mods.toml, etc.).
    /// Populated when Modrinth hash lookup provides no match.
    pub local_mod_data: Option<String>,
    /// Absolute path of the cached extracted icon for unmatched content
    /// files. An empty string marks a file that was checked but has no icon.
    pub icon_path: Option<String>,
}
