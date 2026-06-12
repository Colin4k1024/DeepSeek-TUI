# Project Context

**项目名**: CodeWhale
**当前任务**: 2026-06-12-tsp-harness-integration
**阶段**: handoff-ready
**更新时间**: 2026-06-12

## Tech Stack

- Rust (Cargo workspace, 16 crates)
- TypeScript / Next.js (Web)
- Node.js (Integrations: Feishu/Telegram bridge)
- ratatui (TUI)
- axum (HTTP/SSE server)
- SQLite (State persistence)

## 当前任务

将 TSP (Team Skills Platform, @colin4k1024/tsp v2.5.1) 双向集成到 CodeWhale：
1. TSP 侧新增 `codewhale-home.js` 安装适配器
2. CodeWhale Rust 侧新增 rules/agents/contexts 加载支持
3. 安装时用 Node.js，运行时零依赖

## 关键依赖

- TSP install-targets adapter 接口稳定性
- CodeWhale skills/commands loader 已验证兼容
- CodeWhale config.toml hooks 格式
- SubAgent Custom type 可配置扩展

## 风险

- 200+ skills 可能影响启动性能（缓解: 延迟解析）
- JS hooks 转 Shell 可能丢失功能（缓解: optional 标记）
- rules 注入可能膨胀 system prompt（缓解: 语言过滤 + token 上限）

## 下一步

- 进入 `/team-execute`，由 backend-engineer 实现 Slice 1 (安装适配器) 和 Slice 2 (Rust 侧扩展)
- qa-engineer 完成 Slice 3 (安装验证)
- tech-lead 完成 Slice 4 (文档)
