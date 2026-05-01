# GameMode Switcher Rust

<div align="center">
  <img src="./icon.png" width="96" height="96" alt="GameMode Switcher Rust icon" />

  <h1>GameMode Switcher Rust</h1>

  <p>
    一个专为 Windows 游戏场景准备的预设切换工具：一键进入游戏模式，一键恢复日常模式。
  </p>

  <p>
    <a href="#功能特性">功能特性</a>
    ·
    <a href="#快速开始">快速开始</a>
    ·
    <a href="#clash-设置">Clash 设置</a>
    ·
    <a href="#本地开发">本地开发</a>
    ·
    <a href="#注意事项">注意事项</a>
  </p>

  <p>
    <img alt="platform" src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D4?style=flat-square" />
    <img alt="tauri" src="https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat-square" />
    <img alt="rust" src="https://img.shields.io/badge/Rust-2021-B7410E?style=flat-square" />
    <img alt="frontend" src="https://img.shields.io/badge/React%20%2B%20TypeScript-20232A?style=flat-square" />
  </p>
</div>

## 简介

GameMode Switcher Rust 是一个 Windows 桌面小工具，用来把“打游戏前后那堆重复操作”整理成可切换的预设。

它不是游戏加速器，也不会让帧率原地起飞。它解决的是更朴素但很烦的问题：进入游戏前关闭无关后台、启动需要的辅助软件、处理 Clash/TUN/系统代理；退出游戏后再把该恢复的东西恢复回来。

## 功能特性

- **预设工作台**：为联机游戏、单机游戏、录制游戏等场景建立不同预设。
- **双向应用编排**：进入游戏模式时可关闭/启动程序，退出游戏模式时可反向恢复/关闭程序。
- **Clash 状态管理**：进入游戏模式前记录 Clash/TUN/Windows 系统代理状态，再按配置关闭；退出时恢复原状态。
- **系统托盘**：主窗口可隐藏到托盘，右键直接开启/关闭当前预设或切换预设。
- **开机自启**：支持注册表 Run 键自启，并可静默启动到托盘。
- **低常驻占用**：基于 Rust + Tauri 实现，日常常驻更轻。

## 使用场景

- 打游戏前需要手动关闭下载器、同步盘、代理、聊天工具等后台。
- 网络游戏环境容易被 TUN 或系统代理影响。
- 退出游戏后还得手动恢复常用软件。
- 不想维护一堆 bat/PowerShell 脚本。

## 快速开始

目前建议从 GitHub Releases 下载构建好的安装包。发布前如果还没有 Release，可以先按下面的本地构建方式运行。

运行后你可以：

1. 新建一个游戏场景预设。
2. 配置进入游戏模式时要关闭/启动的程序。
3. 配置退出游戏模式时要恢复/关闭的程序。
4. 如果使用 Clash，在全局设置里填好 Clash 路径、控制端口和 API 密钥。
5. 把窗口关到托盘，通过托盘菜单切换模式。

## 技术栈

- Tauri v2
- Rust
- React
- TypeScript
- Vite

## 平台支持

当前仅支持 Windows 10/11。

程序会使用 Windows 进程、注册表、WinINet 和托盘相关能力。部分操作需要管理员权限，普通启动时默认会尝试提权；如果需要禁用提权，可使用 `--no-elevate` 参数。

## 本地开发

环境要求：

- Rust toolchain
- Node.js
- pnpm

启动开发环境：

```powershell
pnpm install
pnpm tauri:dev
```

## 构建

```powershell
pnpm install
pnpm tauri:build
```

构建产物位于：

```text
src-tauri/target/release/bundle/
```

## 配置文件

Rust 版本使用新的配置文件：

```text
%APPDATA%\GameModeSwitcherRust\config_v2.json
```

旧 Python 版本的 `config.json` 不会自动迁移。

## Clash 设置

如果要管理 Clash，需要在全局设置里配置：

- Clash 可执行路径
- Clash 控制端口
- API 密钥（如果你的 Clash 控制 API 设置了 secret）

工具会尽量通过 Clash API 修改 TUN/系统代理状态，并记录进入游戏模式前的状态用于恢复。

## 常用脚本

- `pnpm dev`：启动 Vite。
- `pnpm build`：TypeScript 检查并构建前端。
- `pnpm tauri:dev`：启动 Tauri 开发模式。
- `pnpm tauri:build`：构建 Tauri 发布包。

## 项目结构

```text
src/                         React 前端
src-tauri/src/app/           Tauri commands 与托盘集成
src-tauri/src/core/domain/   核心模型与错误模型
src-tauri/src/core/services/ 配置、模式编排、进程、Clash、自启服务
src-tauri/src/infra/clash/   Clash HTTP 客户端
src-tauri/src/infra/windows/ Windows 平台能力
```

## 注意事项

- 这不是游戏加速器，只负责环境切换。
- 关闭进程前请确认对应程序可以被安全退出。
- 代理状态恢复依赖进入游戏模式时记录到的状态。
- 如果系统在游戏模式开启期间重启，程序会在下次启动时自动清理运行态标记。

## License

发布前请补充许可证文件，例如 MIT、Apache-2.0 或 GPL-3.0。
