//! 断点续传：工作区匹配 + 检查点读写，原子写。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mod_translation::analyze::{
    JarInspection, reload_structured_templates,
};
use crate::mod_translation::error::{Result, TranslateError};

pub const RESUME_FILE: &str = ".mod-translator-resume.json";
pub const CHECKPOINT_FILE: &str = ".mod-translator-checkpoint.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeMarker {
    pub version: u32,
    pub input_hash: String,
    pub resume_identity: String,
    pub created_at: String,
    pub inspection: JarInspection,
}

/// 在工作区根目录找匹配 input_hash + resume_identity 的 job 目录。
pub fn find_resumable_workspace(
    workspace_root: &Path,
    input_hash: &str,
    identity: &str,
) -> Result<Option<PathBuf>> {
    let entries = std::fs::read_dir(workspace_root).map_err(|error| {
        TranslateError::io("unable to read workspace root", error)
    })?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("job-") {
            continue;
        }
        let marker_path = path.join(RESUME_FILE);
        let Ok(content) = std::fs::read_to_string(&marker_path) else {
            continue;
        };
        let Ok(marker) = serde_json::from_str::<ResumeMarker>(&content) else {
            continue;
        };
        if marker.input_hash == input_hash && marker.resume_identity == identity
        {
            matches.push((marker.created_at, path));
        }
    }
    matches.sort_by(|left, right| right.0.cmp(&left.0));
    Ok(matches.into_iter().next().map(|(_, path)| path))
}

pub fn read_resume_marker(directory: &Path) -> Option<ResumeMarker> {
    let content = std::fs::read_to_string(directory.join(RESUME_FILE)).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_resume_marker(
    directory: &Path,
    marker: &ResumeMarker,
) -> Result<()> {
    let content = format!(
        "{}\n",
        serde_json::to_string(marker).map_err(|error| {
            TranslateError::config(format!(
                "resume marker serialization: {error}"
            ))
        })?
    );
    std::fs::write(directory.join(RESUME_FILE), content).map_err(|error| {
        TranslateError::io("unable to write resume marker", error)
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub version: u32,
    pub task_id: String,
    pub research_completed: bool,
    pub research_summary: String,
    pub completed_language_batches: Vec<String>,
    pub class_exclusions: Vec<String>,
    pub class_replacement_count: usize,
    pub class_changed_files: Vec<String>,
    #[serde(default)]
    pub work_graph: Option<serde_json::Value>,
    #[serde(default)]
    pub event_cursor: u64,
    #[serde(default)]
    pub transport_handoffs: u32,
    #[serde(default)]
    pub last_verified_weight: f64,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub harness: Option<serde_json::Value>,
}

impl Checkpoint {
    pub fn fresh(task_id: String) -> Self {
        Self {
            version: 5,
            task_id,
            stage: "autonomous".to_string(),
            ..Default::default()
        }
    }
}

pub fn read_checkpoint(directory: &Path) -> Option<Checkpoint> {
    let content =
        std::fs::read_to_string(directory.join(CHECKPOINT_FILE)).ok()?;
    serde_json::from_str::<Checkpoint>(&content)
        .ok()
        .filter(|checkpoint| (1..=5).contains(&checkpoint.version))
}

pub fn save_checkpoint(
    directory: &Path,
    checkpoint: &Checkpoint,
) -> Result<()> {
    let content = format!(
        "{}\n",
        serde_json::to_string(checkpoint).map_err(|error| {
            TranslateError::config(format!("checkpoint serialization: {error}"))
        })?
    );
    let path = directory.join(CHECKPOINT_FILE);
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, content).map_err(|error| {
        TranslateError::io("unable to write checkpoint", error)
    })?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        TranslateError::io("unable to move checkpoint into place", error)
    })
}

/// 把当前工作图/账本/记忆写进检查点（同步，供自主循环每轮落盘）。
pub fn update_checkpoint_from_state(
    checkpoint: &mut Checkpoint,
    work_graph: &crate::mod_translation::ledger::WorkGraph,
    class_ledger: &crate::mod_translation::ledger::ClassDecisionLedger,
    memory: &crate::mod_translation::ledger::TaskMemory,
    stage: &str,
) {
    checkpoint.stage = stage.to_string();
    checkpoint.class_exclusions = class_ledger.snapshot_exclusions();
    checkpoint.class_replacement_count = class_ledger.replacement_count;
    checkpoint.class_changed_files = class_ledger.replaced_files.clone();
    checkpoint.work_graph = Some(
        serde_json::to_value(work_graph.snapshot())
            .unwrap_or(serde_json::Value::Null),
    );
    checkpoint.harness =
        Some(serde_json::to_value(memory).unwrap_or(serde_json::Value::Null));
}

/// 恢复时把工作区里已有的翻译成果合并回检查点语义。
pub fn prepare_resumed_inspection(
    workspace: &Path,
    inspection: &mut JarInspection,
) -> Result<()> {
    reload_structured_templates(workspace, &mut inspection.language_sources);
    // 从磁盘读回已写出的目标文件，合并进 existing_target，避免恢复后重复翻译。
    for source in &mut inspection.language_sources {
        let target = workspace.join(&source.target_path);
        if !target.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&target).unwrap_or_default();
        let current = crate::mod_translation::writeback::read_language_target(
            &content, source,
        );
        source.existing_target = current;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_translation::analyze::{LanguageKind, LanguageSource};
    use crate::mod_translation::ledger::{
        ClassDecisionLedger, WorkGraph, WorkKind,
    };

    #[test]
    fn resume_marker_matches_by_hash_and_identity() {
        let dir = tempfile::tempdir().unwrap();
        let job = dir.path().join("job-test");
        std::fs::create_dir_all(&job).unwrap();
        let inspection = JarInspection {
            input_path: PathBuf::from("C:/mods/demo.jar"),
            original_filename: "demo.jar".to_string(),
            loader: crate::mod_translation::analyze::Loader::Fabric,
            mod_ids: vec!["demo".to_string()],
            project_names: vec!["Demo".to_string()],
            mod_version: Some("1.0.0".to_string()),
            minecraft_version_range: None,
            contained_mods: Vec::new(),
            signed: false,
            total_entries: 1,
            uncompressed_bytes: 10,
            language_sources: Vec::new(),
            class_candidates: Vec::new(),
            resource_coverage: Vec::new(),
            warnings: Vec::new(),
        };
        let marker = ResumeMarker {
            version: 2,
            input_hash: "abc123".to_string(),
            resume_identity: "mod-translator-v2".to_string(),
            created_at: "now".to_string(),
            inspection,
        };
        write_resume_marker(&job, &marker).unwrap();

        let found =
            find_resumable_workspace(dir.path(), "abc123", "mod-translator-v2")
                .unwrap()
                .expect("matching workspace should be found");
        assert_eq!(found, job);
        assert!(
            find_resumable_workspace(dir.path(), "other", "mod-translator-v2")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn newest_matching_workspace_is_selected() {
        let dir = tempfile::tempdir().unwrap();
        for (name, created_at) in [
            ("job-old", "2026-08-07T01:00:00Z"),
            ("job-new", "2026-08-07T02:00:00Z"),
        ] {
            let job = dir.path().join(name);
            std::fs::create_dir_all(&job).unwrap();
            write_resume_marker(
                &job,
                &ResumeMarker {
                    version: 3,
                    input_hash: "abc123".to_string(),
                    resume_identity: "mod-translator-v3".to_string(),
                    created_at: created_at.to_string(),
                    inspection: JarInspection {
                        input_path: PathBuf::from("C:/mods/demo.jar"),
                        original_filename: "demo.jar".to_string(),
                        loader: crate::mod_translation::analyze::Loader::Fabric,
                        mod_ids: vec!["demo".to_string()],
                        project_names: Vec::new(),
                        mod_version: None,
                        minecraft_version_range: None,
                        contained_mods: Vec::new(),
                        signed: false,
                        total_entries: 0,
                        uncompressed_bytes: 0,
                        language_sources: Vec::new(),
                        class_candidates: Vec::new(),
                        resource_coverage: Vec::new(),
                        warnings: Vec::new(),
                    },
                },
            )
            .unwrap();
        }
        assert_eq!(
            find_resumable_workspace(dir.path(), "abc123", "mod-translator-v3")
                .unwrap(),
            Some(dir.path().join("job-new"))
        );
    }

    #[test]
    fn checkpoint_round_trips_and_restores_state() {
        let dir = tempfile::tempdir().unwrap();
        let mut checkpoint = Checkpoint::fresh("TASK-test".to_string());
        let mut graph = WorkGraph::new("TASK-test".to_string());
        graph.upsert(
            WorkKind::Language,
            "goal",
            "assets/x/lang/zh_cn.json#block.x",
            2.0,
        );
        graph.reconcile(
            &graph
                .by_source(
                    WorkKind::Language,
                    "assets/x/lang/zh_cn.json#block.x",
                )
                .unwrap()
                .id,
            true,
            "done",
        );
        let ledger = ClassDecisionLedger::default();
        let memory = crate::mod_translation::ledger::TaskMemory::default();
        update_checkpoint_from_state(
            &mut checkpoint,
            &graph,
            &ledger,
            &memory,
            "test",
        );
        save_checkpoint(dir.path(), &checkpoint).unwrap();

        let restored =
            read_checkpoint(dir.path()).expect("checkpoint should restore");
        assert_eq!(restored.version, 5);
        assert_eq!(restored.stage, "test");
        let restored_graph = serde_json::from_value::<
            crate::mod_translation::ledger::WorkGraphSnapshot,
        >(restored.work_graph.unwrap())
        .unwrap();
        let graph = WorkGraph::from_snapshot(restored_graph);
        let item = graph
            .by_source(WorkKind::Language, "assets/x/lang/zh_cn.json#block.x")
            .unwrap();
        assert_eq!(
            item.status,
            crate::mod_translation::ledger::WorkStatus::Verified
        );
    }

    #[test]
    fn resumed_inspection_merges_existing_target_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets/x/lang")).unwrap();
        std::fs::write(
            dir.path().join("assets/x/lang/en_us.json"),
            "{\"a\":\"Iron\",\"b\":\"Gold\"}",
        )
        .unwrap();
        // 恢复时磁盘上已存在 zh_cn.json（上次中断前只写好了 a）
        std::fs::write(
            dir.path().join("assets/x/lang/zh_cn.json"),
            "{\"a\":\"铁\"}",
        )
        .unwrap();
        let mut inspection = JarInspection {
            input_path: PathBuf::from("C:/mods/demo.jar"),
            original_filename: "demo.jar".to_string(),
            loader: crate::mod_translation::analyze::Loader::Fabric,
            mod_ids: vec!["demo".to_string()],
            project_names: vec![],
            mod_version: None,
            minecraft_version_range: None,
            contained_mods: Vec::new(),
            signed: false,
            total_entries: 1,
            uncompressed_bytes: 1,
            language_sources: vec![LanguageSource {
                kind: LanguageKind::Json,
                namespace: "x".to_string(),
                source_path: "assets/x/lang/en_us.json".to_string(),
                target_path: "assets/x/lang/zh_cn.json".to_string(),
                entries: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert("a".to_string(), "Iron".to_string());
                    map.insert("b".to_string(), "Gold".to_string());
                    map
                },
                existing_target: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert("a".to_string(), "铁".to_string());
                    map
                },
                structured_template: None,
                localized_layout: None,
            }],
            class_candidates: Vec::new(),
            resource_coverage: Vec::new(),
            warnings: Vec::new(),
        };
        prepare_resumed_inspection(dir.path(), &mut inspection).unwrap();
        // b 没有译文 → 仍需翻译；a 已有中文 → 不再要求
        let required = inspection.language_sources[0].required_keys();
        assert_eq!(required, vec!["b"]);
        assert_eq!(
            inspection.language_sources[0]
                .existing_target
                .get("a")
                .map(String::as_str),
            Some("铁")
        );
        assert!(inspection.language_sources[0].structured_template.is_none());
    }
}
