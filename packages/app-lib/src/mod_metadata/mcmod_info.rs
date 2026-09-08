use serde::Deserialize;

/// Legacy Forge mcmod.info — JSON array of mod entries.
#[derive(Debug, Deserialize)]
pub(crate) struct McmodInfoEntry {
    pub modid: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
    pub url: Option<String>,
    /// Minecraft version targeted by this mod (single version, not a range).
    pub mcversion: Option<String>,
}
