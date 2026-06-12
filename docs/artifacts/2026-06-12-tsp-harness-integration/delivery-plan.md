# Delivery Plan: TSP 双向集成到 CodeWhale

**状态**: draft
**日期**: 2026-06-12
**Owner**: tech-lead
**阶段**: plan → handoff-ready

---

## 版本目标

- **里程碑**: CodeWhale v0.9.x TSP Integration
- **范围**: TSP 作为 CodeWhale 第 12 个安装目标 + CodeWhale 原生兼容 TSP 格式
- **放行标准**: 一键安装成功率 100%，Skills/Commands/Hooks 均可正常加载和执行

---

## 需求挑战会结论

### 假设 1: Skills 格式兼容性

- **质疑人**: architect
- **假设**: TSP SKILL.md 格式与 CodeWhale skills loader 兼容
- **结论**: ✅ 已验证兼容 — CodeWhale `parse_skill()` 原生支持 `---` frontmatter (name/description) + markdown body。TSP 技能可直接放入 `~/.codewhale/skills/` 被发现和加载。
- **残余风险**: TSP 部分技能使用 `trigger`、`allowed-tools` 等扩展字段，CodeWhale 当前只解析 `name` 和 `description`，扩展字段会被忽略但不会导致加载失败。

### 假设 2: Commands 注册机制

- **质疑人**: backend-engineer
- **假设**: TSP 的 slash commands (.md) 可通过目录部署自动注册
- **结论**: ✅ 已验证 — CodeWhale `user_commands` 模块支持从 `~/.codewhale/commands/` 扫描 `.md` 文件，解析 frontmatter 元数据，作为 slash command 注册。TSP 的 `commands/*.md` 可直接部署。
- **残余风险**: 无。TSP commands 使用相同的 frontmatter + body 格式。

### 假设 3: Hooks 运行时不依赖 Node

- **质疑人**: tech-lead
- **假设**: TSP JS hooks 可以不依赖 Node.js 在 CodeWhale 中运行
- **结论**: ⚠️ 部分成立 — CodeWhale hooks 通过 `sh -c command` 执行。JS hooks 需要 `node` 在 PATH 中。策略：关键 hooks 转为 Shell 脚本；非关键 hooks 标记为 optional（有 Node 才启用）。
- **替代路径**: 保留 JS 格式但安装时检测 Node 可用性，不可用时跳过安装这些 hooks。

---

## Brownfield 上下文快照

### CodeWhale 现有机制

| 机制 | 路径/格式 | 状态 |
|------|-----------|------|
| Skills | `~/.codewhale/skills/<name>/SKILL.md` | 成熟，frontmatter + body |
| Commands | `~/.codewhale/commands/<name>.md` | 成熟，frontmatter + body |
| Hooks | `config.toml [[hooks.hooks]]` | 成熟，shell command |
| Agents | SubAgent tool (Custom type) | 成熟，prompt 驱动 |
| Rules | 无独立机制，通过 skills/prompts 注入 | 需扩展 |

### TSP 能力清单（安装目标需覆盖）

| 能力 | 源路径 | 安装目标路径 |
|------|--------|-------------|
| Skills (200+) | `skills/*/SKILL.md` | `~/.codewhale/skills/` |
| Role Agents (8) | `agents/roles/*.md` | `~/.codewhale/agents/` |
| Specialist Agents (30+) | `agents/specialists/*.md` | `~/.codewhale/agents/` |
| Commands (20+) | `commands/*.md` | `~/.codewhale/commands/` |
| Rules | `rules/**/*.md` | `~/.codewhale/rules/` (新增) |
| Hooks | `hooks/*.js` + `hooks.json` | `config.toml` hooks 段 |
| Contexts | `contexts/*.md` | `~/.codewhale/contexts/` (新增) |

---

## Story Slice 列表

### Slice 1: TSP 安装适配器 (codewhale-home.js)

**目标**: 在 TSP 侧实现 codewhale install target
**验收标准**: `node scripts/install-apply.js --target codewhale --profile team` 成功部署全量资产
**依赖**: 无
**Owner**: backend-engineer
**Handoff**: 安装成功 → Slice 3 验证

| 子任务 | 说明 |
|--------|------|
| 1.1 | 创建 `scripts/lib/install-targets/codewhale-home.js` |
| 1.2 | 在 `registry.js` 注册适配器 |
| 1.3 | 定义目标路径映射规则 |
| 1.4 | 实现 Skills/Commands/Agents/Rules/Contexts 文件复制 |
| 1.5 | 实现 Hooks 转换（JS → Shell wrapper 或 config.toml 注入） |
| 1.6 | 安装完成后生成 manifest.json 记录 |

### Slice 2: CodeWhale Rust 侧扩展 (原生 TSP 支持)

**目标**: CodeWhale 运行时原生理解 TSP 扩展概念
**验收标准**: agents/ 目录被扫描，rules/ 注入 system prompt
**依赖**: Slice 1 部署的文件格式确定
**Owner**: backend-engineer
**Handoff**: 代码变更 → Slice 4 测试

| 子任务 | 说明 |
|--------|------|
| 2.1 | Skills loader 增加 `~/.codewhale/agents/` 扫描路径（agents 当作特殊 skills 加载） |
| 2.2 | 增加 `~/.codewhale/rules/` 目录，启动时注入 system prompt |
| 2.3 | 增加 `~/.codewhale/contexts/` 支持（可选上下文文件） |
| 2.4 | SubAgent Custom type 从 agents/ 目录读取 prompt 定义 |

### Slice 3: 安装验证与测试

**目标**: 端到端验证安装流程
**验收标准**: CI 级别自动化测试通过
**依赖**: Slice 1
**Owner**: qa-engineer
**Handoff**: 测试通过 → release-ready

| 子任务 | 说明 |
|--------|------|
| 3.1 | 编写 `test-codewhale-install.js` 安装测试 |
| 3.2 | 验证 skills 加载数量和名称 |
| 3.3 | 验证 commands 注册数量 |
| 3.4 | 验证 hooks 配置注入 |
| 3.5 | 回归测试：安装后 CodeWhale 启动时间 |

### Slice 4: 文档与用户指南

**目标**: 用户可自助完成集成
**验收标准**: README 中有 CodeWhale 安装说明
**依赖**: Slice 1 + 2
**Owner**: tech-lead

---

## 角色分工

| 角色 | 负责 Slice | 交接对象 |
|------|-----------|----------|
| tech-lead | 编排 + Slice 4 | 全体 |
| architect | 映射规则设计 | backend-engineer |
| backend-engineer | Slice 1 + Slice 2 | qa-engineer |
| qa-engineer | Slice 3 | tech-lead |

---

## 风险与依赖清单

| # | 风险 | 概率 | 影响 | 缓解 |
|---|------|------|------|------|
| 1 | TSP 200+ skills 导致 CodeWhale 启动慢 | 中 | 启动退化 >500ms | profile 模式按需安装，延迟发现 |
| 2 | 部分 TSP skills 引用其他 skills（交叉依赖） | 低 | 功能不完整 | 安装时依赖分析，确保完整性 |
| 3 | Hooks JS→Shell 转换丢失逻辑 | 中 | Hook 行为降级 | 仅转换无状态 hooks，有状态的标记 optional |
| 4 | CodeWhale rules/ 注入影响 token 预算 | 中 | system prompt 膨胀 | 按语言/框架选择性加载 |

---

## 检查节点

| 节点 | 条件 | Owner |
|------|------|-------|
| 方案评审 | architect 映射方案确认 | tech-lead |
| 开发完成 | Slice 1+2 代码合入 | backend-engineer |
| 测试完成 | Slice 3 全通过 | qa-engineer |
| 发布准备 | 文档就绪 + 无 P0 阻塞 | tech-lead |

---

## 技能装配清单

| 类型 | 技能 | 触发原因 | 主责角色 |
|------|------|----------|----------|
| shared | agent-harness-construction | 安装目标适配器设计 | architect |
| shared | coding-standards | 映射脚本质量 | backend-engineer |
| shared | karpathy-guidelines | 范围收敛护栏 | tech-lead |
| ecc | rust-patterns | Rust 侧扩展实现 | backend-engineer |

---

## Implementation Readiness 结论

| 维度 | 状态 | 证据 |
|------|------|------|
| 需求挑战会 | ✅ 完成 | 3 个核心假设已验证，结论记录在上方 |
| 格式兼容性 | ✅ 确认 | CodeWhale parse_skill() 源码验证 |
| 目标路径 | ✅ 确认 | 5 个映射点已明确 |
| 运行时依赖 | ✅ 策略确定 | 安装时 Node / 运行时不依赖 |
| 前置阻塞 | 无 | 可立即开始 |

**结论**: Implementation-ready，可进入 `/team-execute`。
