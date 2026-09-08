use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLink {
    Unmanaged,
    ModrinthModpack {
        project_id: String,
        version_id: String,
    },
    /// A CurseForge modpack managed by project/file IDs.
    CurseForgeModpack {
        project_id: String,
        version_id: String,
    },
    ServerProject {
        project_id: String,
    },
    /// A server project that points at a separate content project/version.
    ServerProjectModpack {
        server_project_id: String,
        content_project_id: String,
        content_version_id: String,
    },
    /// A custom modpack source without a Modrinth project/version link.
    ImportedModpack {
        project_id: Option<String>,
        version_id: Option<String>,
        name: Option<String>,
        version_number: Option<String>,
        filename: Option<String>,
    },
    SharedInstance {
        shared_instance_id: Uuid,
    },
}
