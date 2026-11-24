# AniShelf — Claude Code 开发需求包

本目录用于把 AniShelf 的产品目标、技术约束、开发阶段和执行规则交给 Claude Code。建议将整个目录内容复制到项目根目录，或先解压到 `D:\Projects\AniShelf`。

## 文件说明

- `CLAUDE.md`：Claude Code 项目级长期规则，建议放在项目根目录。
- `PRODUCT_REQUIREMENTS.md`：产品需求与 V1.0 范围。
- `TECHNICAL_DESIGN.md`：技术架构、数据库、文件管理、安全与关键实现约束。
- `DEVELOPMENT_PLAN.md`：分阶段开发任务与验收标准。
- `PROMPTS.md`：可逐阶段复制给 Claude Code 的执行指令。
- `ENVIRONMENT_SETUP.md`：Windows 本地开发环境检查与配置指引。

## 推荐使用方式

1. 在 Windows 安装 Git、Node.js LTS、Rust stable、Visual Studio C++ Build Tools 等必要工具。
2. 创建项目目录，例如 `D:\Projects\AniShelf`。
3. 将本目录中的文件复制到该目录根部。
4. 在 PowerShell 中进入项目目录，运行 `claude`。
5. 首先发送 `PROMPTS.md` 中的“阶段 0：环境检查与项目初始化”指令。
6. 每完成一个阶段，先检查验收标准、运行测试并提交 Git，再开始下一阶段。

## 重要约定

Claude Code 可以在本地项目目录中读写文件并执行命令，但安装系统级依赖、修改系统配置、执行高风险命令前必须先说明并征求确认。不要让它未经确认就删除文件、覆盖用户数据或放宽系统权限。
