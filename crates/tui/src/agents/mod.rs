//! Agent definition discovery for TSP-compatible agent definitions.
//!
//! Scans `~/.codewhale/agents/` for markdown agent definition files.
//! Each `.md` file defines a role or specialist agent that can be
//! referenced by the SubAgent Custom type.

use std::fs;
use std::path::{Path, PathBuf};

use crate::logging;

#[must_use]
pub fn default_agents_dir() -> PathBuf {
    dirs::home_dir().map_or_else(
        || PathBuf::from("/tmp/codewhale/agents"),
        |p| p.join(".codewhale").join("agents"),
    )
}

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub path: PathBuf,
    pub kind: AgentKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentKind {
    Role,
    Specialist,
}

#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: Vec<AgentDefinition>,
}

impl AgentRegistry {
    /// Discover agent definitions from the given directory.
    ///
    /// Expects flat `.md` files. Files prefixed with `specialist-` are
    /// classified as specialist agents; all others as role agents.
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
                    "Failed to read agents directory {}: {err}",
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

            let (kind, name) = if let Some(specialist_name) = stem.strip_prefix("specialist-") {
                (AgentKind::Specialist, specialist_name.to_string())
            } else {
                (AgentKind::Role, stem.to_string())
            };

            let description = extract_first_heading(&content)
                .unwrap_or_else(|| name.clone());

            registry.agents.push(AgentDefinition {
                name,
                description,
                prompt: content,
                path,
                kind,
            });
        }

        registry.agents.sort_by(|a, b| a.name.cmp(&b.name));
        registry
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&AgentDefinition> {
        self.agents.iter().find(|a| a.name == name)
    }

    #[must_use]
    pub fn roles(&self) -> Vec<&AgentDefinition> {
        self.agents.iter().filter(|a| a.kind == AgentKind::Role).collect()
    }

    #[must_use]
    pub fn specialists(&self) -> Vec<&AgentDefinition> {
        self.agents.iter().filter(|a| a.kind == AgentKind::Specialist).collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    #[must_use]
    pub fn list_names(&self) -> Vec<&str> {
        self.agents.iter().map(|a| a.name.as_str()).collect()
    }
}

fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix('#') {
            let heading = heading.trim_start_matches('#').trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_role_and_specialist_agents() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("tech-lead.md"),
            "# Tech Lead\n\nYou are a tech lead.",
        )
        .unwrap();
        fs::write(
            tmp.path().join("specialist-code-reviewer.md"),
            "# Code Reviewer\n\nReview code for quality.",
        )
        .unwrap();

        let registry = AgentRegistry::discover(tmp.path());
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.roles().len(), 1);
        assert_eq!(registry.specialists().len(), 1);

        let lead = registry.find_by_name("tech-lead").unwrap();
        assert_eq!(lead.kind, AgentKind::Role);
        assert_eq!(lead.description, "Tech Lead");

        let reviewer = registry.find_by_name("code-reviewer").unwrap();
        assert_eq!(reviewer.kind, AgentKind::Specialist);
    }
}
