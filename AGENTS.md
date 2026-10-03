# AgyOrbit · Agent 指引

公开仓库（MIT）：在菜单栏 / 系统托盘切换 Google Antigravity 账号并查看各账号额度。Tauri 2：Rust 后端负责全部副作用，React 前端只渲染后端快照。本文件是各编码 Agent 共用的仓库级规则；`CLAUDE.md` 导入本文件。人类贡献流程见 [`CONTRIBUTING.md`](CONTRIBUTING.md)，不在这里复制。

优先级：用户当次指令 > 本文件 > 其它文档。代码和 [`docs/antigravity-integration.md`](docs/antigravity-integration.md) 里的实测记录是事实依据。

## 按需加载

| 要做的事 | 先读 |
|---|---|
| 改代码、找模块 | [`docs/architecture.md`](docs/architecture.md) |
| 涉及 Antigravity 的存储、OAuth、接口、进程 | [`docs/antigravity-integration.md`](docs/antigravity-integration.md) |
| 改界面或交互 | [`docs/design.md`](docs/design.md) |
| 发版、打包、官网 `site/` | [`docs/release.md`](docs/release.md) |
| README / 官网截图 | [`docs/design.md`](docs/design.md) 的 Screenshots 一节 |

## 硬约束

- **唯一 owner。** 一个事实一个文件，其它地方链接。版本号只在 `src-tauri/Cargo.toml`；`README.md` 与 `README.zh-CN.md` 同步更新，不让两份说法漂移。命令名与事件名只在 `src/ipc.ts`，前后端数据结构以 `src-tauri/src/model.rs` 为准、`src/types.ts` 跟随。
- **不碰机密。** 令牌、OAuth client secret、真实邮箱、个人路径不进代码、测试、日志、文档、截图和 issue。client secret 只在运行时从本机 Antigravity 读取，禁止写进仓库（也不要写出 `GOCSPX-` 形状的字面量）。
- **先实测，再落文档。** 依赖 Antigravity 行为的改动，先在真实安装上用只读系统测试验证，再更新 `docs/antigravity-integration.md` 的验证版本表。没验证过的平台写明 not verified，不把推断写成事实。
- **副作用归后端。** 前端不持有令牌、不推断后端已知的状态；新增行为先在 Rust 里找到 owner 模块，命令层只做转发。
- **破坏性操作先确认、可回滚。** 结束 Antigravity、改写登录凭据都必须经用户在界面上确认；写入失败要恢复原凭据。
- **支持面。** macOS 26+（只用 Liquid Glass，不做旧系统回退）与 Windows 10/11 x64。不加兼容分支、旧入口或空占位。
- **单文件规模。** 一文件一职责，≤800 行；超过先按职责拆分。

## 协作与发版

- **不直接推 `main`。** "Protect main" ruleset 对所有人生效（含维护者）：变更走主题分支和 PR；交付前检查完整 diff 与 `git diff --check`。
- **PR 先开 draft（CI 跳过）。** 本机门禁通过、审查收口后转 ready；CI 在 macOS 与 Windows 上运行，汇总检查 `gate` 通过且分支包含最新 `main` 才能合入，默认 squash 合并。
- **发版手动触发。** 版本号与 `CHANGELOG.md` 的版本段随普通 PR 合入 `main`；之后在 Actions 手动运行 `Release` 并输入同一版本号，它只从 `main` 构建、生成 draft release，核对产物后再发布。步骤见 [`docs/release.md`](docs/release.md)。

## 门禁

改动后跑与 CI 相同的检查：

```bash
bun run check
bun run build
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

本机装有并登录了 Antigravity 时，相关改动加跑只读系统测试：`cd src-tauri && cargo test system_tests -- --ignored --nocapture`。行为变化在 `CHANGELOG.md` 的 `[Unreleased]` 记一笔。

## 自主权与交付

在上述边界内默认放权，可以大改；只在真实的范围变更、需要在用户机器上实际切换账号或结束进程、合入 `main` 或发版，或需要只有用户能给的输入（浏览器授权、第二个账号）时停下来。工作区里已有的改动属于用户，不覆盖、不回滚、不混进本次提交。中文回复，标识符与命令保持英文；交付说明改了什么、跑了哪些检查、哪些没验证。
