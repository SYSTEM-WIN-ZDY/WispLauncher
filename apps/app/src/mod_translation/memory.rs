//! 翻译记忆：跨模组缓存同源文本，JSON 文件落盘 + LRU 淘汰。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mod_translation::error::{Result, TranslateError};
use crate::mod_translation::quality::{has_chinese, validate_protected_tokens};

const MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryEntry {
    translation: String,
    updated_at: u64,
    hits: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryFile {
    version: u32,
    entries: HashMap<String, MemoryEntry>,
}

#[derive(Debug)]
pub struct TranslationMemory {
    path: PathBuf,
    entries: HashMap<String, MemoryEntry>,
    dirty: bool,
}

impl TranslationMemory {
    pub async fn load(path: PathBuf) -> Self {
        let read_path = path.clone();
        let entries = tokio::task::spawn_blocking(move || {
            let Ok(content) = std::fs::read_to_string(&read_path) else {
                return HashMap::new();
            };
            serde_json::from_str::<MemoryFile>(&content)
                .map(|file| file.entries)
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();
        Self {
            path,
            entries,
            dirty: false,
        }
    }

    pub fn memory_key(
        mod_ids: &[String],
        namespace: &str,
        source: &str,
    ) -> String {
        let mut ids = mod_ids.to_vec();
        ids.sort();
        ids.dedup();
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&ids).unwrap_or_default());
        hasher.update(namespace.to_ascii_lowercase().as_bytes());
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 命中必须再过占位符 + 中文校验，不过就当 miss。
    pub fn lookup(
        &mut self,
        mod_ids: &[String],
        namespace: &str,
        source: &str,
    ) -> Option<String> {
        let key = Self::memory_key(mod_ids, namespace, source);
        let entry = self.entries.get_mut(&key)?;
        let translation = entry.translation.trim().to_string();
        if translation.is_empty()
            || !has_chinese(&translation)
            || validate_protected_tokens(source, &translation).is_some()
        {
            return None;
        }
        entry.hits += 1;
        entry.updated_at = now_seconds();
        self.dirty = true;
        Some(translation)
    }

    pub fn record(
        &mut self,
        mod_ids: &[String],
        namespace: &str,
        source: &str,
        translation: &str,
    ) {
        let key = Self::memory_key(mod_ids, namespace, source);
        self.entries.insert(
            key,
            MemoryEntry {
                translation: translation.to_string(),
                updated_at: now_seconds(),
                hits: 0,
            },
        );
        self.dirty = true;
    }

    pub async fn flush(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        if self.entries.len() > MAX_ENTRIES {
            let mut entries = self.entries.drain().collect::<Vec<_>>();
            entries.sort_by(|left, right| {
                right
                    .1
                    .updated_at
                    .cmp(&left.1.updated_at)
                    .then_with(|| right.1.hits.cmp(&left.1.hits))
            });
            entries.truncate(MAX_ENTRIES);
            self.entries = entries.into_iter().collect();
        }
        let entries = self.entries.clone();
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    TranslateError::io(
                        "unable to create translation memory directory",
                        error,
                    )
                })?;
            }
            let file = MemoryFile {
                version: 1,
                entries,
            };
            let content = format!(
                "{}\n",
                serde_json::to_string(&file).map_err(|error| {
                    TranslateError::config(format!(
                        "translation memory serialization: {error}"
                    ))
                })?
            );
            let temporary = path.with_extension("json.tmp");
            std::fs::write(&temporary, content).map_err(|error| {
                TranslateError::io("unable to write translation memory", error)
            })?;
            std::fs::rename(&temporary, &path).map_err(|error| {
                TranslateError::io(
                    "unable to move translation memory into place",
                    error,
                )
            })
        })
        .await
        .map_err(|error| {
            TranslateError::config(format!(
                "translation memory flush task: {error}"
            ))
        })??;
        self.dirty = false;
        Ok(())
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

pub fn memory_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join("translation-memory-v1.json")
}
