//! Optional context files that can be loaded into the system prompt.
//!
//! Scans `~/.codewhale/contexts/` for markdown context definition files.
//! These provide additional system prompt content that can be selectively
//! activated by the user (e.g. `dev`, `review`, `research` modes).

use std::fs;
use std::path::{Path, PathBuf};

use crate::logging;

#[must_use]
pub fn default_contexts_dir() -> PathBuf {
    dirs::home_dir().map_or_else(
        || PathBuf::from("/tmp/codewhale/contexts"),
        |p| p.join(".codewhale").join("contexts"),
    )
}

#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    pub content: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct ContextRegistry {
    contexts: Vec<Context>,
}

impl ContextRegistry {
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
                    "Failed to read contexts directory {}: {err}",
                    dir.display()
                ));
                return registry;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().map_or(true, |ext| ext != "md") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            registry.contexts.push(Context {
                name: stem.to_string(),
                content,
                path,
            });
        }

        registry.contexts.sort_by(|a, b| a.name.cmp(&b.name));
        registry
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Context> {
        self.contexts.iter().find(|c| c.name == name)
    }

    #[must_use]
    pub fn list_names(&self) -> Vec<&str> {
        self.contexts.iter().map(|c| c.name.as_str()).collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_context_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("dev.md"), "Development context.").unwrap();
        fs::write(tmp.path().join("review.md"), "Review context.").unwrap();
        fs::write(tmp.path().join("notes.txt"), "Not a context.").unwrap();

        let registry = ContextRegistry::discover(tmp.path());
        assert_eq!(registry.len(), 2);
        assert!(registry.find_by_name("dev").is_some());
        assert!(registry.find_by_name("review").is_some());
        assert!(registry.find_by_name("notes").is_none());
    }
}
