# Windows 本地开发环境准备

## 1. 目标

为 AniShelf（Tauri 2 + React + TypeScript + Vite + Rust）准备 Windows 原生开发环境。

推荐 Windows 原生 PowerShell 开发，不需要一开始使用 WSL2。

## 2. 必要组件

| 组件 | 用途 | 检查命令 |
|---|---|---|
| Git for Windows | 版本管理 | `git --version` |
| Node.js LTS | 前端运行时 | `node -v` |
| npm | Node 包管理 | `npm -v` |
| pnpm | 项目包管理 | `pnpm -v` |
| Rust stable | Tauri 后端 | `rustc --version` |
| Cargo | Rust 构建 | `cargo --version` |
| Visual Studio C++ Build Tools | MSVC 编译工具链 | 在 Visual Studio Installer 中检查 |
| Windows SDK | Windows 编译依赖 | 在 Visual Studio Installer 中检查 |
| WebView2 Runtime | Tauri WebView | Windows 应用运行时检查 |

官方参考：
- Claude Code 安装与使用：https://code.claude.com/docs/en/overview
- Tauri 2 前置条件：https://v2.tauri.app/start/prerequisites/
- Rust 安装：https://www.rust-lang.org/tools/install
- Node.js：https://nodejs.org/
- Git：https://git-scm.com/downloads
- Visual Studio Build Tools：https://visualstudio.microsoft.com/visual-cpp-build-tools/

## 3. 安装 Claude Code

PowerShell 安装命令（执行前确认来源和网络环境）：

```powershell
irm https://claude.ai/install.ps1 | iex
```

验证：

```powershell
claude --version
```

## 4. 安装 pnpm

Node.js 安装完成后：

```powershell
corepack enable
corepack prepare pnpm@latest --activate
pnpm --version
```

如果 Corepack 不可用，可使用官方 pnpm 安装说明处理，不要在不清楚来源时运行第三方脚本。

## 5. 安装 Rust

按照 Rust 官方安装器安装 stable toolchain。安装完成后重新打开 PowerShell：

```powershell
rustc --version
cargo --version
rustup show
```

## 6. Visual Studio C++ Build Tools

在 Visual Studio Installer 中安装 C++ 桌面开发相关组件，至少确认：
- MSVC x64/x86 Build Tools；
- Windows 10 或 Windows 11 SDK；
- C++ CMake tools（若项目构建链需要）。

具体工作负载名称可能随 Installer 版本变化，以 Tauri 官方 prerequisites 为准。

## 7. 环境检查脚本

可在 PowerShell 逐项执行：

```powershell
git --version
node -v
npm -v
pnpm -v
rustc --version
cargo --version
rustup show
```

再检查 Visual Studio Build Tools 与 WebView2 Runtime。

## 8. 安全注意

- 安装系统级组件可能需要管理员权限；
- 不要未经确认修改系统 PATH、注册表或执行清理命令；
- 安装完成后重新打开终端；
- 不要把 API Token、个人路径或用户数据提交到 Git；
- Claude Code 执行命令前应说明高风险操作并请求确认。
