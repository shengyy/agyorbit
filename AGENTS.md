# AgyOrbit · Agent 指引

公开仓库（MIT）：在菜单栏 / 系统托盘切换 Google Antigravity 账号并查看各账号额度。Tauri 2：Rust 后端负责全部副作用，React 前端只渲染后端快照。本文件是各编码 Agent 共用的仓库级规则，只放每次任务都成立的边界；`CLAUDE.md` 导入本文件。人类贡献流程见 [`CONTRIBUTING.md`](CONTRIBUTING.md)，不在这里复制。

优先级：用户当次指令 > 本文件 > 其它文档。代码、测试和 [`docs/antigravity-integration.md`](docs/antigravity-integration.md) 的实测记录是已实现行为的事实，文档定义意图与边界；冲突时先查明哪一侧过期，在同一变更里更新唯一 owner。

## 按需加载

只读当前任务需要的上下文。触达下列范围前先读对应 owner；完整文档地图、目录职责与写入规则见 [`docs/README.md`](docs/README.md)：

| 范围 | 先读 |
|---|---|
| 产品范围、规则、非目标 | [`PRODUCT.md`](PRODUCT.md) |
| 当前平台与已验证事实 | [`docs/status.md`](docs/status.md) |
| 代码结构、数据流、切换时序、存储 | [`docs/architecture.md`](docs/architecture.md) |
| Antigravity 的凭据、OAuth、接口、进程 | [`docs/antigravity-integration.md`](docs/antigravity-integration.md) |
| 界面呈现、视觉、图标、截图 | [`docs/design.md`](docs/design.md) |
| 版本、发版、签名、官网 | [`docs/release.md`](docs/release.md) |
| 待办、挂账、待确认 | GitHub Issues：挂账标 `deferred`（写明暂缓原因与触发条件），待确认标 `needs-decision`；不在文档里列清单 |

改了什么就跑哪条检查（CI 在 macOS 与 Windows 上跑全部，精确命令归 `package.json` 与 [`CONTRIBUTING.md`](CONTRIBUTING.md)）：

- 只改文档 → `git diff --check`，核对链接与 owner 路由。
- `src/`、`site/`、`scripts/` → `bun run check`、`bun run build`。
- `src-tauri/` → 在 `src-tauri/` 跑 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（需先有 `bun run build` 的产物）。
- 依赖 Antigravity 行为 → 另在真实安装上跑只读系统测试 `cargo test system_tests -- --ignored --nocapture`。
- 界面外观变化 → `bun scripts/screenshot/render.ts` 重渲截图。

## 工程卫生（硬约束）

- **唯一 owner。** 一个事实一个文件，其它地方链接。版本号只在 `src-tauri/Cargo.toml`；命令名与事件名只在 `src/ipc.ts`；前后端数据结构以 `src-tauri/src/model.rs` 为准、`src/types.ts` 跟随；`README.md` 与 `README.zh-CN.md` 同步更新。
- **禁补丁、禁兼容。** 修到根因，不在下游加分支绕过；替代方案落地后，在同一变更里删掉旧路径、旧名称及其测试和文档，不留双轨。旧版本留下的本机状态不写迁移代码，在 `CHANGELOG.md` 写明手动处理方式。
- **单文件与目录规模。** 单文件 ≤800 行正常；800–1200 行按职责考虑拆分；>1200 行在触碰时拆分，或在文件头写明原因。判据是职责，不是行数；锁文件与生成物不计。一个目录一个职责，新目录同时在 [`docs/README.md`](docs/README.md) 登记。
- **不过度工程。** 默认选状态最少、owner 最少、能完整验证的方案；新增依赖、抽象或后台流程需要真实需求，非目标见 [`PRODUCT.md`](PRODUCT.md)。
- **验证按改动校准。** 先跑能证伪本次改动的最窄检查，再按影响面扩大；切换、凭据写入、结束进程这类路径要同时覆盖成功与失败回滚。

## 本仓不变量

- 副作用归后端：前端不持有令牌，不推断后端已知的状态；新增行为先在 Rust 里找到 owner 模块，命令层只做转发。
- 不碰机密：令牌、OAuth client secret、真实邮箱、个人路径不进代码、测试、日志、文档、截图和 issue。client secret 只在运行时从本机 Antigravity 读取，禁止写进仓库，也不写出 `GOCSPX-` 形状的字面量。截图只用 `scripts/screenshot/` 的虚构账号渲染。
- 先实测再落文档：依赖 Antigravity 或平台行为的改动，先在真实安装上验证，再更新 `docs/antigravity-integration.md` 与 `docs/status.md`；没验证过的写明 not verified，不把推断写成事实。
- 破坏性操作先确认、可回滚：结束 Antigravity、改写登录凭据都必须经用户在界面上确认；写入失败要恢复原凭据。
- 支持面只有 macOS 26+ 与 Windows 10/11 x64，不做旧系统回退。macOS 包以 Bundle ID 为代码标识 ad-hoc 签名，登录项依赖它，不得改回链接器默认签名。

## 自主权

在上述边界内默认放权：重构、依赖增删、测试策略、目录调整、删除过时路径、可逆的本机 Git 操作，都不必逐项请示。工作区里已有的改动属于用户，不覆盖、不回滚、不混进本次提交。只在三种情况停下来：破坏性或不可逆动作（在用户机器上实际切换账号、结束进程或改写凭据，发布 release，改动仓库规则）、真实的范围变更、只有用户能给的输入（浏览器授权、第二个账号）。停下之前先把已授权的部分做完。用户在描述问题或提问、而不是要求改动时，交付物是你的判断。

## 交付

- 中文回复；标识符、命令、路径保持英文。只报告能对应到本次工具结果的事实：失败就贴输出，跳过就说明跳过。
- 不直接推 `main`：仓库的 "Protect main" ruleset 对所有人生效。变更走主题分支和 PR；交付前检查完整 diff 与 `git diff --check`。
- PR 先开 draft（CI 跳过）；本机检查通过、审查收口后转 ready，汇总检查 `gate`（macOS + Windows）通过且分支包含最新 `main` 才合入，默认 squash。
- 用户可见的变化在 `CHANGELOG.md` 的 `[Unreleased]` 记一笔；已验证事实变化时同步 `docs/status.md`。
- 发版手动触发：版本号与 `CHANGELOG.md` 版本段随普通 PR 合入后，在 Actions 运行 `Release`，核对 draft 后发布，步骤见 [`docs/release.md`](docs/release.md)。
