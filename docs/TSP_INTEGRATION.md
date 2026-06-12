# TSP (Team Skills Platform) Integration Guide

## Overview

CodeWhale supports integration with TSP (@colin4k1024/tsp), a role-based team skills platform providing 200+ skills, 30+ specialist agents, 20+ slash commands, and runtime hooks.

## Quick Start

### Prerequisites

- Node.js >= 18 (install-time only, not needed at runtime)
- CodeWhale installed and configured

### Installation

```bash
# Clone TSP repository
git clone https://github.com/Colin4k1024/tsp.git
cd tsp

# Install to CodeWhale with team profile
node scripts/install-apply.js --target codewhale --profile team
```

### What Gets Installed

| Component | Target Path | Count |
|-----------|-------------|-------|
| Skills | `~/.codewhale/skills/` | 200+ |
| Commands | `~/.codewhale/commands/` | 20+ |
| Agents | `~/.codewhale/agents/` | 38 (8 roles + 30 specialists) |
| Rules | `~/.codewhale/rules/` | 88 files |
| Contexts | `~/.codewhale/contexts/` | 3 |
| Hooks | `~/.codewhale/hooks/` | 4 |

## Usage

### Skills

After installation, TSP skills are automatically discovered by CodeWhale. Use them via slash commands:

```
/tdd          # Test-Driven Development workflow
/code-review  # Comprehensive code review
/plan         # Implementation planning
/pua          # Enforcement mode for stuck tasks
```

### Agents

TSP agents are available as SubAgent Custom types:

- **Role agents**: tech-lead, architect, frontend-engineer, backend-engineer, etc.
- **Specialist agents**: code-reviewer, security-reviewer, tdd-guide, etc.

### Rules

Rules are automatically injected into the system prompt based on detected project language:

- `common/` — Always included
- `rust/` — Rust projects
- `typescript/` — TypeScript/JavaScript projects
- `python/` — Python projects
- `java/` — Java/Spring projects
- `golang/` — Go projects

### Hooks

Installed hooks provide runtime enhancements:

- `harness-statusline.js` — Status line with context info
- `harness-context-monitor.js` — Context window monitoring
- `harness-prompt-guard.js` — Prompt safety checks

## Profiles

| Profile | Description |
|---------|-------------|
| `team` | Full team workflow (roles + commands + skills) |
| `full` | Everything including all language rules |
| `minimal` | Core skills and commands only |

## Uninstallation

```bash
# Remove all TSP-installed files
node scripts/install-apply.js --target codewhale --uninstall
```

The uninstall reads `.codewhale/ecc-install-state.json` to precisely remove only managed files.

## Architecture

```
TSP Repository
├── scripts/lib/install-targets/codewhale-home.js  ← Install adapter
├── skills/          → ~/.codewhale/skills/
├── commands/        → ~/.codewhale/commands/
├── agents/          → ~/.codewhale/agents/
├── rules/           → ~/.codewhale/rules/
└── hooks/           → ~/.codewhale/hooks/

CodeWhale Runtime (Rust)
├── crates/tui/src/skills/mod.rs      ← Discovers SKILL.md files
├── crates/tui/src/rules/mod.rs       ← Discovers & injects rules
├── crates/tui/src/agents/mod.rs      ← Discovers agent definitions
├── crates/tui/src/contexts/mod.rs    ← Discovers context files
└── crates/tui/src/commands/user_commands.rs ← Loads .md commands
```

## Compatibility

- CodeWhale skills loader natively supports TSP `SKILL.md` format (frontmatter + body)
- Commands use the same frontmatter + markdown body format
- No Node.js runtime dependency — all assets are static markdown files
- Hooks can run with `node` if available, gracefully degrade if not
