//! Rules discovery and system prompt injection.
//!
//! Scans `~/.codewhale/rules/` for markdown rule files organized by
//! language/category. At startup, rules matching the detected project
//! language are injected into the system prompt.

use std::fs;
use std::path::{Path, PathBuf};

use crate::logging;

const MAX_RULES_INJECTION_CHARS: usize = 8_000;

#[must_use]
pub fn default_rules_dir() -> PathBuf {
    dirs::home_dir().map_or_else(
        || PathBuf::from("/tmp/codewhale/rules"),
        |p| p.join(".codewhale").join("rules"),
    )
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub namespace: String,
    pub filename: String,
    pub content: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct RulesRegistry {
    rules: Vec<Rule>,
}

impl RulesRegistry {
    /// Discover rules from the given directory.
    ///
    /// Expected layout:
    /// ```text
    /// rules/
    /// ├── common/
    /// │   ├── coding-style.md
    /// │   └── testing.md
    /// ├── rust/
    /// │   └── patterns.md
    /// └── typescript/
    ///     └── frontend.md
    /// ```
    #[must_use]
    pub fn discover(dir: &Path) -> Self {
        let mut registry = Self::default();
        if !dir.is_dir() {
            return registry;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                logging::warn(&format!(
                    "Failed to read rules directory {}: {err}",
                    dir.display()
                ));
                return registry;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                let namespace = name.to_string();
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file()
                            && sub_path
                                .extension()
                                .is_some_and(|ext| ext == "md")
                        {
                            if let Ok(content) = fs::read_to_string(&sub_path) {
                                let filename = sub_path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                registry.rules.push(Rule {
                                    namespace: namespace.clone(),
                                    filename,
                                    content,
                                    path: sub_path,
                                });
                            }
                        }
                    }
                }
            } else if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    registry.rules.push(Rule {
                        namespace: "root".to_string(),
                        filename: name.to_string(),
                        content,
                        path,
                    });
                }
            }
        }

        registry.rules.sort_by(|a, b| {
            a.namespace
                .cmp(&b.namespace)
                .then_with(|| a.filename.cmp(&b.filename))
        });
        registry
    }

    /// Filter rules by project languages and return system prompt injection text.
    #[must_use]
    pub fn system_prompt_injection(&self, project_languages: &[&str]) -> String {
        let mut output = String::new();
        let mut total_chars = 0;

        // Always include "common" namespace
        let relevant: Vec<&Rule> = self
            .rules
            .iter()
            .filter(|r| {
                r.namespace == "common"
                    || r.namespace == "root"
                    || project_languages.iter().any(|lang| r.namespace == *lang)
            })
            .collect();

        for rule in &relevant {
            let entry = format!(
                "\n## Rule: {}/{}\n\n{}\n",
                rule.namespace, rule.filename, rule.content
            );
            if total_chars + entry.len() > MAX_RULES_INJECTION_CHARS {
                break;
            }
            output.push_str(&entry);
            total_chars += entry.len();
        }

        output
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    #[must_use]
    pub fn namespaces(&self) -> Vec<String> {
        let mut ns: Vec<String> = self
            .rules
            .iter()
            .map(|r| r.namespace.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        ns.sort();
        ns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_rules_from_nested_dirs() {
        let tmp = TempDir::new().unwrap();
        let common_dir = tmp.path().join("common");
        fs::create_dir_all(&common_dir).unwrap();
        fs::write(common_dir.join("style.md"), "# Style\nBe consistent.").unwrap();

        let rust_dir = tmp.path().join("rust");
        fs::create_dir_all(&rust_dir).unwrap();
        fs::write(rust_dir.join("patterns.md"), "# Patterns\nUse Result.").unwrap();

        let registry = RulesRegistry::discover(tmp.path());
        assert_eq!(registry.len(), 2);
        assert!(registry.namespaces().contains(&"common".to_string()));
        assert!(registry.namespaces().contains(&"rust".to_string()));
    }

    #[test]
    fn filters_by_language() {
        let tmp = TempDir::new().unwrap();
        let common_dir = tmp.path().join("common");
        fs::create_dir_all(&common_dir).unwrap();
        fs::write(common_dir.join("style.md"), "common rule").unwrap();

        let rust_dir = tmp.path().join("rust");
        fs::create_dir_all(&rust_dir).unwrap();
        fs::write(rust_dir.join("ownership.md"), "rust rule").unwrap();

        let ts_dir = tmp.path().join("typescript");
        fs::create_dir_all(&ts_dir).unwrap();
        fs::write(ts_dir.join("react.md"), "ts rule").unwrap();

        let registry = RulesRegistry::discover(tmp.path());
        let injection = registry.system_prompt_injection(&["rust"]);
        assert!(injection.contains("common rule"));
        assert!(injection.contains("rust rule"));
        assert!(!injection.contains("ts rule"));
    }
}
