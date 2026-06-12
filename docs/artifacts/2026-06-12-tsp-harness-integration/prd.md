# PRD: 集成 TSP (Team Skills Platform) 到 CodeWhale

**状态**: draft
**日期**: 2026-06-12
**Owner**: tech-lead
**阶段**: intake

---

## 背景

CodeWhale 是基于 Rust 的 AI 编码助手平台，具备 Skills 插件、Hooks 生命周期、MCP 协议扩展和 SubAgent 编排等核心能力。

TSP (Team Skills Platform, @colin4k1024/tsp v2.5.1) 是公司级角色化团队技能平台，提供 200+ Skills、30+ Specialist Agents、20+ Slash Commands、Runtime Hooks 和多平台安装工具链，已支持 11 个安装目标。

当前需要将 TSP 的能力体系双向集成到 CodeWhale 中。

### 业务问题

- CodeWhale 缺少结构化的团队协作工作流（角色分工、交接契约、质量门禁）
- CodeWhale Skills 生态较薄，无法覆盖企业级多语言多框架场景
- 需要对齐公司级规范（编码风格、安全基线、测试策略等）

### 触发原因

- 公司技术规范统一需要
- CodeWhale 用户对团队协作能力的需求

### 当前约束

- CodeWhale 核心为 Rust 单仓，运行时不应引入 Node.js 硬依赖
- TSP Skills 格式为 `SKILL.md` (markdown + frontmatter)
- CodeWhale 已有自己的 skills 加载路径 `~/.codewhale/skills/`

---

## 目标与成功标准

### 业务目标

- CodeWhale 获得完整的企业级团队协作工作流能力
- 200+ TSP Skills 可在 CodeWhale 会话中按需加载
- 安装过程一键完成，运行时零额外依赖

### 用户价值

- 开发者获得开箱即用的代码审查、TDD、安全扫描等专业工作流
- 团队获得标准化的需求 intake → 方案设计 → 实现 → 测试 → 发布链路
- 通过 specialist agents 获得多角色协作视角

### 成功指标

| 指标 | 目标值 |
|------|--------|
| TSP install 到 codewhale 成功率 | 100% |
| Skills 加载成功率 | >= 95%（部分技能可能有平台限制） |
| Hooks 执行延迟 | < 500ms per hook |
| 集成后 CodeWhale 启动时间退化 | < 100ms |

---

## 用户故事

### US-1: 一键安装 TSP 到 CodeWhale

**作为** CodeWhale 用户
**我想** 通过一条命令安装 TSP 全量能力
**以便于** 立即获得团队级技能和工作流

**验收标准:**
- `node scripts/install-apply.js --target codewhale --profile team` 成功执行
- `~/.codewhale/skills/` 下出现 TSP 技能目录
- `~/.codewhale/agents/` 或等效路径下出现角色定义
- CodeWhale config.toml 中注入 hooks 配置

### US-2: 在 CodeWhale TUI 中使用 TSP 技能

**作为** 开发者
**我想** 在 CodeWhale 会话中输入 `/tdd` 或 `/code-review`
**以便于** 触发 TSP 定义的标准工作流

**验收标准:**
- 斜杠命令列表中可见 TSP 注册的命令
- 命令执行后按 TSP 定义的 SKILL.md prompt 引导 agent 行为
- 命令不依赖 Node.js 运行时

### US-3: CodeWhale 原生解析 TSP SKILL.md 格式

**作为** 技能贡献者
**我想** CodeWhale 直接理解 TSP 的 SKILL.md 格式
**以便于** 无需格式转换就能使用 TSP 生态的技能

**验收标准:**
- CodeWhale Rust skills loader 识别 SKILL.md frontmatter (name, trigger, description)
- 支持 TSP 的目录约定 (SKILL.md + references/ + examples/)
- 兼容 CodeWhale 原有技能格式

---

## 范围

### In Scope

1. **TSP 侧**: 新增 `codewhale-home.js` 安装目标适配器
2. **CodeWhale 侧**: Rust skills loader 扩展支持 TSP SKILL.md 格式
3. **映射层**: Skills/Agents/Hooks/Commands 的双向映射规则
4. **安装验证**: 端到端安装测试脚本
5. **文档**: 集成指南和配置说明

### Out of Scope

- CodeWhale Web 前端改动
- VS Code 扩展改动
- TSP 工作流引擎 (workflow-state, artifact-persistence) — Phase 2
- TSP 的 memory/observation 运行时能力 — Phase 2
- 修改 TSP 现有技能内容

---

## 技术决策

### 集成方式: 双向集成

- TSP 侧新增 `codewhale-home.js` install target adapter
- CodeWhale Rust 侧添加 TSP SKILL.md 格式原生解析支持

### 运行时依赖: 安装时 Node，运行时不依赖

- 安装阶段使用 Node.js 运行 `install-apply.js`
- Hooks 转为纯 Shell 脚本或 CodeWhale 原生 hook 格式
- Skills/Commands 以文件形式部署，由 Rust 加载器消费

---

## 风险与依赖

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| CodeWhale skills loader 格式不完全兼容 TSP | 部分技能加载失败 | 实现兼容层，必要时做格式适配 |
| TSP JS hooks 转 Shell 可能丢失功能 | hook 行为降级 | 保留关键 hooks，非关键降级为 noop |
| CodeWhale SubAgent type 枚举为编译期固定 | agents 映射受限 | 走 Custom type + 配置文件扩展 |
| 安装后 skills 数量过多影响启动 | TUI 启动变慢 | 按 profile 选择性安装，延迟加载 |

### 关键依赖

- CodeWhale `crates/tui/src/skills/` 加载器的扩展能力
- TSP install-targets/registry.js 的适配器接口
- CodeWhale config.toml hooks 格式的稳定性

---

## 待确认项

| # | 待确认项 | 影响 | Owner | 状态 |
|---|----------|------|-------|------|
| 1 | CodeWhale skills loader 是否支持 frontmatter 解析 | 格式兼容方案 | architect | pending |
| 2 | Hooks event 名称映射表 | hooks 桥接实现 | architect | pending |
| 3 | SubAgent Custom type 的配置扩展机制 | agents 集成深度 | backend-engineer | pending |
| 4 | Commands 注册是否支持目录扫描 | commands 映射 | backend-engineer | pending |
| 5 | TSP profile "team" 包含的具体模块列表 | 默认安装范围 | tech-lead | pending |
