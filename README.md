# GameMode Switcher Rust

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
