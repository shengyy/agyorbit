<p align="center">
  <img src="assets/brand/app-icon.svg" width="112" alt="AgyOrbit 图标">
</p>

<h1 align="center">AgyOrbit</h1>

<p align="center">
  多个 Google Antigravity 账号，点一下就切换。<br>
  在菜单栏看清每个账号的额度，不用退出重登。
</p>

<p align="center">
  <a href="README.md">English</a> | <strong>简体中文</strong>
</p>

<p align="center">
  <a href="https://github.com/shengyy/agyorbit/actions/workflows/ci.yml"><img src="https://github.com/shengyy/agyorbit/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/shengyy/agyorbit/releases"><img src="https://img.shields.io/github/v/release/shengyy/agyorbit?include_prereleases&sort=semver" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

---

在 [Antigravity](https://antigravity.google) 里用多个 Google 账号时，每次切换都要退出、重新登录、再等待，
而且你不知道哪个账号还剩 Claude 或 Gemini 额度。AgyOrbit 常驻 macOS 菜单栏或 Windows 通知区域，把这两件事一起解决。

## 功能

- **所有账号一眼看清。** 每个账号的 Gemini 和 Claude（Opus、Sonnet、GPT-OSS）额度，5 小时窗口与本周窗口并排显示，
  附重置时间；余量最多、值得切过去的账号会被标出来。
- **确认一次即可切换。** AgyOrbit 关闭 Antigravity 和 `agy` 命令行，把 Antigravity 登录到所选账号，校验无误后重新打开。
  任何一步失败都会恢复原来的登录。
- **在浏览器里添加账号。** 普通的 Google 登录页，不用在 Antigravity 里退出；Antigravity 当前已登录的账号会自动纳入。
- **重启或结束 Antigravity** 也在同一处完成，确认框会列出将要关闭的进程。
- **原生体验。** macOS 上是 Liquid Glass 弹出面板，支持浅色 / 深色模式、英文和简体中文。
- **隐私。** 令牌只存在系统钥匙串里，只发给 Google。没有服务器，没有统计上报。

## 系统要求

| 平台 | 状态 |
|---|---|
| macOS 26 及以上（Apple 芯片与 Intel） | 支持 |
| Windows 10 / 11 x64 | 实验性：CI 可构建，尚未在真实安装上验证 |

需要已安装 Antigravity：AgyOrbit 从本机安装中读取登录配置。

## 安装

从 [Releases](https://github.com/shengyy/agyorbit/releases) 下载最新的 `.dmg`（macOS）或 `-setup.exe`（Windows）。

安装包暂未做代码签名。macOS 首次打开请右键 → **打开**，或执行
`xattr -dr com.apple.quarantine /Applications/AgyOrbit.app`；Windows 在 SmartScreen 中选择 **更多信息 → 仍要运行**。

从源码构建见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 使用

1. 启动 AgyOrbit 并点击图标，Antigravity 当前登录的账号会标为 **当前**。
2. 点 **添加账号**，浏览器会打开 Google 登录页，选择另一个账号并允许访问。
3. 点击某个账号（或它的 **切换** 按钮）并确认，Antigravity 会以该账号重新启动。
4. 右键账号可重新授权、复制邮箱或移除；顶部 `⋯` 菜单里有开机启动、日志和退出。

## 工作原理

Antigravity 把登录凭据存在系统钥匙串（另有一份回退文件）。AgyOrbit 使用从本机安装中读取的 Antigravity 自有 OAuth client
完成浏览器登录，因此拿到的凭据与 Antigravity 自己登录时存下的完全一致；切换就是在 Antigravity 关闭期间改写这份凭据。
额度数据来自 IDE 使用的同一组 Cloud Code 接口。细节及验证版本见
[docs/antigravity-integration.md](docs/antigravity-integration.md)。

## 隐私与合理使用

AgyOrbit 只负责切换官方 Antigravity 应用使用的是 **你自己的** 哪个账号：不代理模型流量、不共享账号、不绕过额度，
除 Google 外不与任何服务通信。请在 Google 条款范围内使用。存储与发送的具体内容见 [SECURITY.md](SECURITY.md)。

AgyOrbit 是独立项目，与 Google 无隶属或背书关系。"Antigravity" 与 "Gemini" 是 Google LLC 的商标。

## 参与贡献

欢迎 issue 和 PR，目前最需要的是 Windows 上的实机验证。先读 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [docs/](docs/README.md)。

## 许可证

[MIT](LICENSE)
