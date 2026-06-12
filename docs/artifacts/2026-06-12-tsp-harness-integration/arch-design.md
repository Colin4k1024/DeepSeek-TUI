# Architecture Design: TSP → CodeWhale 集成

**状态**: draft
**日期**: 2026-06-12
**Owner**: architect
**阶段**: plan

---

## 系统边界

```
┌─────────────────────────────────────────────────────────────────┐
│                    TSP (harness-public)                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ skills/  │ │ agents/  │ │commands/ │ │  hooks/  │  ...      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└────────────────────────┬────────────────────────────────────────┘
                         │ install-apply.js --target codewhale
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│              Install Adapter (codewhale-home.js)                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  文件映射 + 格式转换 + config.toml 注入 + manifest 记录  │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────────┘
                         │ 部署到 ~/.codewhale/
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                 CodeWhale Runtime (Rust)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Skills   │ │ Commands │ │  Hooks   │ │ Agents   │  ...      │
│  │ Loader   │ │  Loader  │ │ Executor │ │ (Custom) │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 组件映射规则

### 1. Skills 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `skills/<name>/SKILL.md` | `~/.codewhale/skills/<name>/SKILL.md` | 直接复制（格式兼容） |
| `skills/<name>/references/` | `~/.codewhale/skills/<name>/references/` | 直接复制 |
| `skills/<name>/examples/` | `~/.codewhale/skills/<name>/examples/` | 直接复制 |

**兼容性**: CodeWhale `SkillRegistry::discover()` 递归扫描 `SKILL.md`，支持嵌套目录布局。TSP 技能无需任何格式转换。

### 2. Commands 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `commands/<name>.md` | `~/.codewhale/commands/<name>.md` | 直接复制 |

**兼容性**: CodeWhale `load_user_commands()` 扫描 `commands/` 目录下的 `.md` 文件，解析 frontmatter。TSP commands 使用相同格式。

### 3. Agents 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `agents/roles/<role>.md` | `~/.codewhale/agents/<role>.md` | 直接复制 |
| `agents/specialists/<spec>.md` | `~/.codewhale/agents/<spec>.md` | 直接复制 |

**消费方式**: CodeWhale SubAgent `Custom` 类型从 agents/ 目录读取 prompt 定义。安装时可生成一个 `agents-index.json` 便于快速查找。

### 4. Rules 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `rules/common/*.md` | `~/.codewhale/rules/common/*.md` | 直接复制 |
| `rules/<lang>/*.md` | `~/.codewhale/rules/<lang>/*.md` | 直接复制 |

**消费方式**: CodeWhale 启动时扫描 `rules/` 目录，将匹配当前项目语言的规则注入 system prompt。（需 Rust 侧新增）

### 5. Hooks 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `hooks/hooks.json` | `config.toml [[hooks.hooks]]` | JSON → TOML 转换 |
| `hooks/*.js` (无状态) | Shell wrapper 或标记 optional | 生成 `sh -c "node <path>"` |

**事件映射表:**

| TSP Hook Event | CodeWhale HookEvent |
|----------------|---------------------|
| PreToolUse | ToolCallBefore |
| PostToolUse | ToolCallAfter |
| Stop | SessionEnd |
| UserPromptSubmit | MessageSubmit |
| SubagentSpawn | SubagentSpawn |
| SubagentComplete | SubagentComplete |

### 6. Contexts 映射

| TSP 源 | CodeWhale 目标 | 转换规则 |
|--------|----------------|----------|
| `contexts/*.md` | `~/.codewhale/contexts/*.md` | 直接复制 |

**消费方式**: 作为可选的 system prompt 扩展源（需 Rust 侧新增支持 `contexts_dir` 配置）。

---

## 关键数据流

### 安装流程

```
1. 用户执行: node install-apply.js --target codewhale --profile team
2. 解析 profile → 确定需要安装的 module IDs
3. codewhale-home.js adapter:
   a. 确定目标根目录: ~/.codewhale/
   b. 遍历模块清单，按映射规则复制文件
   c. 转换 hooks.json → 合并到 config.toml
   d. 生成 .install-manifest.json 记录安装状态
4. 完成输出: 安装了 N skills, M commands, K agents, J rules
```

### 运行时加载流程 (CodeWhale 侧)

```
1. TUI 启动 → config.toml 读取
2. SkillRegistry::discover(~/.codewhale/skills/) → 200+ skills
3. load_user_commands(~/.codewhale/commands/) → 20+ commands  
4. HookDispatcher 加载 config.toml hooks 段 → hooks 就绪
5. [新增] RulesLoader::discover(~/.codewhale/rules/) → 按语言过滤注入
6. [新增] AgentLoader::discover(~/.codewhale/agents/) → SubAgent Custom 定义
```

---

## 接口约定

### codewhale-home.js Adapter 接口

```javascript
module.exports = {
  target: 'codewhale',
  label: 'CodeWhale (~/.codewhale/)',
  supports(id) { return id === 'codewhale'; },
  
  resolveTargetDir(options) {
    return path.join(os.homedir(), '.codewhale');
  },
  
  scaffoldDirs(targetDir) {
    return ['skills', 'commands', 'agents', 'rules', 'contexts'];
  },
  
  mapSkill(sourcePath, targetDir) { /* ... */ },
  mapCommand(sourcePath, targetDir) { /* ... */ },
  mapAgent(sourcePath, targetDir) { /* ... */ },
  mapRule(sourcePath, targetDir) { /* ... */ },
  mapHook(hookDef, configTomlPath) { /* ... */ },
};
```

### CodeWhale Rust 侧新增模块

```rust
// crates/tui/src/rules/mod.rs (新增)
pub struct RulesRegistry {
    rules: Vec<Rule>,
}

impl RulesRegistry {
    pub fn discover(dir: &Path, project_languages: &[&str]) -> Self;
    pub fn system_prompt_injection(&self) -> String;
}
```

---

## 技术选型

| 决策点 | 选择 | 原因 |
|--------|------|------|
| 安装器语言 | Node.js (已有) | TSP 安装工具链成熟，无需重写 |
| Rust 侧扩展方式 | 新增 module 文件 | 最小改动，不影响现有 skills/hooks |
| config.toml 合并 | 追加式 (不覆盖已有 hooks) | 安全，用户自定义不丢失 |
| 大量 skills 性能 | 延迟解析 (只读 frontmatter) | 启动仅扫描目录 + 文件名 |

---

## 风险与约束

| 风险 | 缓解 |
|------|------|
| 200+ skills 启动扫描慢 | 文件名缓存 + 延迟解析 body |
| rules 注入导致 system prompt token 膨胀 | 按项目语言过滤，上限 2000 tokens |
| config.toml hooks 段冲突 | 安装前备份，使用唯一 name 前缀 `tsp-*` |
| 卸载不干净 | manifest.json 记录安装文件，支持精确卸载 |
