//! 模组名分层解析：内嵌 → 已知表 → 生成 → 直译 → 文件名 → 显示名 → modId。

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

const GENERIC_NAMES_RE: &str = r"^(?:(?:中文|魔法|冒险|科技|装饰|工具|未知|未命名)?模组|未命名|未知|有趣的冒险|中文扩展|未命名扩展)$";

const KNOWN_NAMES: &[(&str, &str)] = &[
    ("macaw'swindows", "Macaw 的窗户"),
    ("mcwwindows", "Macaw 的窗户"),
    ("iron'sspells'nspellbooks", "铁魔法与法术书"),
    ("irons_spellbooks", "铁魔法与法术书"),
    ("farmer'sdelight", "农夫乐事"),
    ("farmersdelight", "农夫乐事"),
    ("alex'scaves", "Alex 的洞穴"),
    ("alexscaves", "Alex 的洞穴"),
    ("mekanism", "通用机械"),
    ("lootmate", "战利品助手"),
];

const WORD_TRANSLATIONS: &[(&str, &str)] = &[
    ("window", "窗户"),
    ("windows", "窗户"),
    ("cave", "洞穴"),
    ("caves", "洞穴"),
    ("spell", "法术"),
    ("spells", "法术"),
    ("spellbook", "法术书"),
    ("spellbooks", "法术书"),
    ("loot", "战利品"),
    ("mate", "助手"),
    ("farmer", "农夫"),
    ("farmers", "农夫"),
    ("delight", "乐事"),
    ("magic", "魔法"),
    ("iron", "铁"),
    ("tools", "工具"),
    ("tool", "工具"),
    ("doors", "门"),
    ("door", "门"),
];

static GENERIC_NAMES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(GENERIC_NAMES_RE).unwrap());
static WORD_MAP: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| WORD_TRANSLATIONS.iter().copied().collect());
static KNOWN_MAP: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| KNOWN_NAMES.iter().copied().collect());

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModNameResult {
    pub name: String,
    pub source: String,
}

fn identity_key(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || *character == '_'
                || *character == '\''
        })
        .collect()
}

pub fn usable_chinese_mod_name(value: &str) -> bool {
    let normalized = value.trim();
    normalized.chars().count() >= 2
        && normalized
            .chars()
            .any(|character| matches!(character, '\u{3400}'..='\u{9fff}'))
        && !GENERIC_NAMES.is_match(normalized)
}

fn original_project_label(original_name: &str) -> Option<String> {
    let stem = Path::new(original_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(original_name)
        .trim()
        .to_string();
    if stem.is_empty()
        || Regex::new(r"^(?:download|mod|file|unknown)(?:[-_ ]?\d+)?$")
            .unwrap()
            .is_match(&stem)
    {
        return None;
    }
    let without_version = Regex::new(
        r"[-_ ]+(?:mc)?\d+\.\d+(?:\.\d+)?(?:[-+._][A-Za-z0-9.]+)*.*$",
    )
    .unwrap()
    .replace_all(&stem, "")
    .trim()
    .to_string();
    let label = (if without_version.is_empty() {
        stem
    } else {
        without_version
    })
    .replace('_', " ")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
    if label.is_empty() { None } else { Some(label) }
}

fn known_chinese_mod_name(
    project_names: &[String],
    mod_ids: &[String],
) -> Option<String> {
    mod_ids
        .iter()
        .chain(project_names.iter())
        .filter_map(|value| KNOWN_MAP.get(identity_key(value).as_str()))
        .map(|value| value.to_string())
        .next()
}

fn translate_english_label(value: &str) -> Option<String> {
    let clean = value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(captures) =
        Regex::new(r"^(.+?)['’]s\s+(.+)$").unwrap().captures(&clean)
    {
        let tail = captures[2]
            .split_whitespace()
            .map(|word| {
                WORD_MAP
                    .get(word.to_ascii_lowercase().as_str())
                    .copied()
                    .unwrap_or(word)
            })
            .collect::<String>();
        if tail
            .chars()
            .any(|character| matches!(character, '\u{3400}'..='\u{9fff}'))
        {
            return Some(format!("{} 的{}", &captures[1], tail));
        }
    }
    let mut translated = false;
    let parts = clean
        .split_whitespace()
        .filter(|word| {
            !Regex::new(r"^(?:mc|forge|fabric|neoforge|mod)$")
                .unwrap()
                .is_match(word)
        })
        .map(|word| {
            if let Some(replacement) =
                WORD_MAP.get(word.to_ascii_lowercase().as_str())
            {
                translated = true;
                replacement.to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("");
    if translated { Some(parts) } else { None }
}

fn translated_fallback(
    project_names: &[String],
    original_name: &str,
) -> Option<ModNameResult> {
    for display_name in project_names {
        if let Some(translated) = translate_english_label(display_name)
            && usable_chinese_mod_name(&translated)
        {
            return Some(ModNameResult {
                name: translated.chars().take(64).collect(),
                source: "translated_display_name".to_string(),
            });
        }
    }
    let original = original_project_label(original_name)?;
    let translated = translate_english_label(&original)?;
    if usable_chinese_mod_name(&translated) {
        Some(ModNameResult {
            name: translated.chars().take(64).collect(),
            source: "translated_filename".to_string(),
        })
    } else {
        None
    }
}

/// 决策链主入口。
pub fn resolve_mod_name(
    project_names: &[String],
    mod_ids: &[String],
    original_name: &str,
    recommended_name: Option<String>,
    recommended_source: Option<String>,
) -> ModNameResult {
    if let Some(embedded) = project_names
        .iter()
        .find(|name| usable_chinese_mod_name(name))
    {
        return ModNameResult {
            name: embedded.trim().chars().take(64).collect(),
            source: "embedded_chinese".to_string(),
        };
    }
    if let Some(name) = recommended_name
        && usable_chinese_mod_name(&name)
    {
        return ModNameResult {
            name: name.trim().chars().take(64).collect(),
            source: recommended_source.unwrap_or_else(|| {
                "researched_or_generated_chinese".to_string()
            }),
        };
    }
    if let Some(known) = known_chinese_mod_name(project_names, mod_ids) {
        return ModNameResult {
            name: known.chars().take(64).collect(),
            source: "known_chinese".to_string(),
        };
    }
    if let Some(translated) = translated_fallback(project_names, original_name)
    {
        return translated;
    }
    if let Some(original) = original_project_label(original_name) {
        return ModNameResult {
            name: original.chars().take(64).collect(),
            source: "original_filename".to_string(),
        };
    }
    if let Some(display_name) = project_names
        .iter()
        .map(|name| name.trim())
        .find(|name| !name.is_empty() && !GENERIC_NAMES.is_match(name))
    {
        return ModNameResult {
            name: display_name.chars().take(64).collect(),
            source: "display_name".to_string(),
        };
    }
    let mod_id = mod_ids.iter().map(|id| id.trim()).find(|id| !id.is_empty());
    ModNameResult {
        name: mod_id.unwrap_or("mod").chars().take(64).collect(),
        source: "mod_id".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_chinese_wins() {
        let result = resolve_mod_name(
            &["Macaw 的窗户".to_string()],
            &["mcwwindows".to_string()],
            "mcwwindows-1.0.jar",
            None,
            None,
        );
        assert_eq!(result.source, "embedded_chinese");
        assert_eq!(result.name, "Macaw 的窗户");
    }

    #[test]
    fn known_names_are_used() {
        let result = resolve_mod_name(
            &[],
            &["mekanism".to_string()],
            "mekanism-1.20.1.jar",
            None,
            None,
        );
        assert_eq!(result.source, "known_chinese");
        assert_eq!(result.name, "通用机械");
    }

    #[test]
    fn possessive_names_translate() {
        let result =
            resolve_mod_name(&[], &[], "Alex's Caves-1.2.jar", None, None);
        assert_eq!(result.source, "translated_filename");
        assert_eq!(result.name, "Alex 的洞穴");
    }

    #[test]
    fn generic_names_fall_through_to_mod_id() {
        let result = resolve_mod_name(
            &[],
            &["weird_mod_id".to_string()],
            "download.jar",
            None,
            None,
        );
        assert_eq!(result.source, "mod_id");
        assert_eq!(result.name, "weird_mod_id");
    }

    #[test]
    fn version_suffixes_are_stripped() {
        assert_eq!(
            original_project_label("some-mod-1.20.1.jar").as_deref(),
            Some("some-mod")
        );
    }
}
