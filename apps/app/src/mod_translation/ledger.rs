//! 翻译工作图、任务记忆与 class 处置账本。

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::mod_translation::analyze;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkKind {
    Language,
    VisibleText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkStatus {
    Pending,
    Claimed,
    Submitted,
    Verified,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub action: String,
    pub outcome: String,
    #[serde(default)]
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub kind: WorkKind,
    pub goal: String,
    pub source: String,
    pub weight: f64,
    #[serde(default)]
    pub attempts: Vec<Attempt>,
    pub status: WorkStatus,
    pub version: u64,
}

impl WorkItem {
    pub fn model_attempt_count(&self) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| {
                matches!(
                    attempt.action.as_str(),
                    "fast_translate"
                        | "deep_translate"
                        | "deep_quality"
                        | "agent"
                )
            })
            .count()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkGraphSnapshot {
    pub task_id: String,
    pub revision: u64,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone)]
pub struct WorkGraph {
    task_id: String,
    items: BTreeMap<String, WorkItem>,
    revision: u64,
}

#[allow(dead_code)]
impl WorkGraph {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            items: BTreeMap::new(),
            revision: 0,
        }
    }

    pub fn from_snapshot(snapshot: WorkGraphSnapshot) -> Self {
        Self {
            task_id: snapshot.task_id,
            items: snapshot
                .items
                .into_iter()
                .map(|item| (item.id.clone(), item))
                .collect(),
            revision: snapshot.revision,
        }
    }

    pub fn snapshot(&self) -> WorkGraphSnapshot {
        WorkGraphSnapshot {
            task_id: self.task_id.clone(),
            revision: self.revision,
            items: self.items.values().cloned().collect(),
        }
    }

    /// 新增或更新一个工作项（按 source 去重）。
    pub fn upsert(
        &mut self,
        kind: WorkKind,
        goal: &str,
        source: &str,
        weight: f64,
    ) -> String {
        let id = work_item_id(&self.task_id, kind, source);
        match self.items.get_mut(&id) {
            Some(item) => {
                item.weight = weight;
                item.goal = goal.to_string();
            }
            None => {
                self.items.insert(
                    id.clone(),
                    WorkItem {
                        id: id.clone(),
                        kind,
                        goal: goal.to_string(),
                        source: source.to_string(),
                        weight,
                        attempts: Vec::new(),
                        status: WorkStatus::Pending,
                        version: 1,
                    },
                );
                self.revision += 1;
            }
        }
        id
    }

    pub fn all(&self) -> Vec<WorkItem> {
        self.items.values().cloned().collect()
    }

    pub fn pending(&self) -> Vec<WorkItem> {
        self.items
            .values()
            .filter(|item| {
                !matches!(
                    item.status,
                    WorkStatus::Verified | WorkStatus::Superseded
                )
            })
            .cloned()
            .collect()
    }

    pub fn by_source(&self, kind: WorkKind, source: &str) -> Option<WorkItem> {
        let id = work_item_id(&self.task_id, kind, source);
        self.items.get(&id).cloned()
    }

    pub fn item(&self, id: &str) -> Option<WorkItem> {
        self.items.get(id).cloned()
    }

    pub fn record_attempt(
        &mut self,
        id: &str,
        action: &str,
        outcome: &str,
        failure_class: Option<&str>,
    ) {
        if let Some(item) = self.items.get_mut(id) {
            item.attempts.push(Attempt {
                action: action.to_string(),
                outcome: outcome.to_string(),
                failure_class: failure_class.map(str::to_string),
            });
            self.revision += 1;
        }
    }

    pub fn reconcile(&mut self, id: &str, accepted: bool, reason: &str) {
        if let Some(item) = self.items.get_mut(id) {
            if accepted {
                item.status = WorkStatus::Verified;
            } else if item.status != WorkStatus::Superseded {
                item.status = WorkStatus::Pending;
            }
            item.version += 1;
            self.revision += 1;
            let _ = reason;
        }
    }

    pub fn reset_for_retry(&mut self, id: &str) {
        if let Some(item) = self.items.get_mut(id) {
            if item.status != WorkStatus::Superseded {
                item.status = WorkStatus::Pending;
            }
            item.attempts.retain(|attempt| {
                !matches!(
                    attempt.action.as_str(),
                    "fast_translate"
                        | "deep_translate"
                        | "deep_quality"
                        | "agent"
                )
            });
            item.version += 1;
            self.revision += 1;
        }
    }

    pub fn supersede(&mut self, id: &str, reason: &str) {
        if let Some(item) = self.items.get_mut(id) {
            item.status = WorkStatus::Superseded;
            item.version += 1;
            self.revision += 1;
            let _ = reason;
        }
    }

    pub fn progress(&self) -> (f64, f64) {
        let total = self.items.values().map(|item| item.weight).sum::<f64>();
        let verified = self
            .items
            .values()
            .filter(|item| {
                matches!(
                    item.status,
                    WorkStatus::Verified | WorkStatus::Superseded
                )
            })
            .map(|item| item.weight)
            .sum::<f64>();
        (verified, total)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

fn work_item_id(task_id: &str, kind: WorkKind, source: &str) -> String {
    let kind_str = match kind {
        WorkKind::Language => "language",
        WorkKind::VisibleText => "visible_text",
    };
    let mut hasher = Sha256::new();
    hasher.update(task_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(kind_str.as_bytes());
    hasher.update(b"\0");
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())[..24].to_string()
}
/// 任务记忆：跨会话继承，各字段有界。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMemory {
    #[serde(default)]
    pub recommended_name: Option<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub glossary: HashMap<String, String>,
    #[serde(default)]
    pub decisions: Vec<Value>,
    #[serde(default)]
    pub uncertainties: Vec<String>,
    #[serde(default)]
    pub discovered_targets: Vec<String>,
    #[serde(default)]
    pub coverage: Option<Value>,
}

impl TaskMemory {
    pub fn update(&mut self, update: Value) {
        if let Some(name) =
            update.get("recommendedName").and_then(Value::as_str)
        {
            self.recommended_name = Some(name.chars().take(80).collect());
        }
        if let Some(summary) = update.get("summary").and_then(Value::as_str) {
            self.summary = truncate(summary, 4_000);
        }
        if let Some(glossary) = update.get("glossary").and_then(Value::as_array)
        {
            for entry in glossary {
                if let (Some(source), Some(translation)) = (
                    entry.get("source").and_then(Value::as_str),
                    entry.get("translation").and_then(Value::as_str),
                ) {
                    self.glossary.insert(
                        truncate(source, 120),
                        truncate(translation, 120),
                    );
                }
            }
            while self.glossary.len() > 500 {
                if let Some(key) = self.glossary.keys().next().cloned() {
                    self.glossary.remove(&key);
                }
            }
        }
        if let Some(decisions) =
            update.get("decisions").and_then(Value::as_array)
        {
            self.decisions.extend(decisions.iter().take(50).cloned());
            while self.decisions.len() > 200 {
                self.decisions.remove(0);
            }
        }
        if let Some(uncertainties) =
            update.get("uncertainties").and_then(Value::as_array)
        {
            for value in uncertainties.iter().filter_map(Value::as_str) {
                self.uncertainties.push(truncate(value, 500));
            }
            while self.uncertainties.len() > 100 {
                self.uncertainties.remove(0);
            }
        }
        if let Some(targets) =
            update.get("discoveredTargets").and_then(Value::as_array)
        {
            for value in targets.iter().filter_map(Value::as_str) {
                self.discovered_targets.push(truncate(value, 500));
            }
            while self.discovered_targets.len() > 500 {
                self.discovered_targets.remove(0);
            }
        }
        if let Some(coverage) = update.get("coverage") {
            self.coverage = Some(coverage.clone());
        }
    }
}

fn truncate(value: &str, maximum: usize) -> String {
    let mut out = value
        .chars()
        .filter(|character| !matches!(*character, '\u{0000}'..='\u{0008}' | '\u{000b}' | '\u{000c}' | '\u{000e}'..='\u{001f}'))
        .collect::<String>();
    out = out.trim().to_string();
    if out.chars().count() > maximum {
        out = out.chars().take(maximum).collect();
    }
    out
}

/// class 候选处置账本（translate.rs 与 ledger.rs 共用）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassDecisionLedger {
    pub decisions: HashMap<String, ClassDecision>,
    pub replaced_files: Vec<String>,
    pub replacement_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDecision {
    pub action: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolutionLedger {
    pub work_graph: WorkGraph,
    pub class_ledger: ClassDecisionLedger,
}

impl ClassDecisionLedger {
    #[allow(dead_code)]
    pub fn is_resolved(&self, id: &str) -> bool {
        self.decisions.contains_key(id)
    }

    pub fn unresolved(
        &self,
        candidates: &[analyze::ClassCandidate],
    ) -> Vec<analyze::ClassCandidate> {
        candidates
            .iter()
            .filter(|candidate| !self.decisions.contains_key(&candidate.id))
            .cloned()
            .collect()
    }

    pub fn snapshot_exclusions(&self) -> Vec<String> {
        self.decisions
            .iter()
            .filter(|(_, decision)| decision.action == "exclude")
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumed_required_work_is_reopened_with_a_fresh_model_budget() {
        let mut graph = WorkGraph::new("TASK-test".to_string());
        let id = graph.upsert(
            WorkKind::Language,
            "translate",
            "assets/demo/lang/zh_cn.json#demo.key",
            1.0,
        );
        graph.record_attempt(&id, "fast_translate", "bad", Some("partial"));
        graph.record_attempt(&id, "quality_audit", "bad", Some("quality"));
        graph.reconcile(&id, true, "old checkpoint");

        graph.reset_for_retry(&id);

        let item = graph.item(&id).unwrap();
        assert_eq!(item.status, WorkStatus::Pending);
        assert_eq!(item.model_attempt_count(), 0);
        assert_eq!(item.attempts.len(), 1);
        assert_eq!(item.attempts[0].action, "quality_audit");
    }
}
