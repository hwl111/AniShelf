<div align="center">
  <img src="src-tauri/icons/128x128.png" width="96" height="96" alt="AniShelf 图标" />
  <h1>AniShelf</h1>
  <p>本地优先的番剧、阅读与笔记管理桌面应用</p>
  <p>
    <img alt="Windows" src="https://img.shields.io/badge/Windows-10%20%2F%2011-0078D4" />
    <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB" />
    <img alt="React" src="https://img.shields.io/badge/React-19-61DAFB" />
    <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-CE422B" />
    <img alt="SQLite" src="https://img.shields.io/badge/SQLite-local--first-003B57" />
  </p>
</div>

> 当前版本：**v1.0.1**（2026-11-24）  
> 安装包请从本仓库的 **Releases** 页面下载。

## 简介

AniShelf 是一个 Windows 优先的个人桌面知识管理应用，把 **番剧管理**、**本地电子书阅读** 和 **知识笔记** 放在一起。数据默认保存在本机，观看记录、阅读、笔记、搜索、收藏与备份在断网时仍然可用。

- 番剧：手动管理或从 Bangumi 导入元数据，记录状态、评分、集数与观看历史
- 阅读：导入 EPUB / PDF / TXT / Markdown，自动保存阅读进度与书签
- 笔记：纯文本笔记、标签、笔记间链接与反向链接、关联番剧/书籍/集数
- 检索：中文全文搜索、收藏页、真实数据统计与活动时间线
- 数据安全：数据库备份/恢复、迁移前自动备份、原始书籍文件只读

## 功能

### 番剧

- 五种观看状态：想看、在看、已看、搁置、弃番
- 评分、个人标签、评论、年份/季度、总集数
- 单集管理、标记已看、补录观看记录与单集感想
- 可选 Bangumi 搜索/导入/刷新；外部元数据不会覆盖个人评分、状态、标签与观看记录

### 书架与阅读器

- EPUB：目录跳转、滚轮翻页、翻页/滚动、字号/行距/主题、阅读进度、书签
- PDF：单页画布渲染、滚轮翻页、页码导航、缩放/适合宽度/页面、进度、书签
- TXT：UTF-8/BOM/GB18030 编码检测、分块读取、连续滚动、进度与书签
- 封面：EPUB 使用内置封面；PDF 自动使用第 1 页；TXT 自动生成首页文字封面
- 原始文件默认只读；可选择复制到应用资料库

### 笔记与关联

- 新建、编辑、删除、搜索与标签筛选
- 笔记之间的显式链接与反向链接面板
- 关联番剧、书籍、番剧集数
- 删除笔记前列出引用影响；删除番剧/书籍/集数时提示受影响的笔记数量

### 搜索、收藏与首页

- 搜索番剧标题/别名/简介、书名/作者、笔记标题/正文与已有批注
- 中文搜索：≥3 字使用 FTS5 trigram，1–2 字使用转义 LIKE 回退
- 收藏页：番剧、书籍、笔记
- 首页：正在追番、最近观看/阅读/笔记、统计与活动时间线（全部来自真实数据库）
- 快捷操作：添加番剧、导入书籍、新建笔记

### 备份与恢复

- 立即备份、另存为、备份列表与元数据
- 恢复前自动备份当前数据库；恢复在重启后生效，失败不会破坏当前数据
- 数据库迁移前自动备份
- 日志写入应用 `logs` 目录

## 下载与安装

1. 打开本仓库的 **Releases** 页面，下载最新的 `AniShelf_x.y.z_x64-setup.exe`。
2. 运行安装包，按当前用户安装，不需要管理员权限。
3. 若 Windows SmartScreen 提示“未知发布者”，请先核对 Release 中的 SHA-256 再决定是否运行。安装包当前未做代码签名。

## 系统要求

- Windows 10 / 11 x64
- Microsoft Edge WebView2 Runtime（Windows 11 通常已内置）
- 建议 300MB 以上可用磁盘空间；书籍文件按实际大小另计

## 数据与隐私

默认数据目录：`%LOCALAPPDATA%\com.anishelf.desktop`

| 内容     | 位置                       |
| -------- | -------------------------- |
| 数据库   | `database\anishelf.sqlite` |
| 封面     | `covers\`                  |
| 备份     | `backups\`                 |
| 日志     | `logs\anishelf.log`        |
| 附件副本 | `attachments\`             |

- 原始书籍文件默认保留在用户选择的位置，AniShelf 不会修改或删除它们。
- 数据库备份**不包含原始书籍文件**；迁移或重装前请单独备份书籍。
- Bangumi Token 仅保存在本机数据库，不写入日志或 Git；不联网时仍可手动管理番剧与观看记录。
- 应用不会主动上传书籍、笔记或个人数据。

## 技术栈

- 桌面：Tauri 2
- 前端：React 19 + TypeScript + Vite + Tailwind CSS v4 + shadcn/ui
- 状态与数据：Zustand、TanStack Query
- 后端：Rust、SQLx、SQLite（WAL、foreign_keys、FTS5 trigram）
- 阅读器：epub.js、PDF.js、Rust 分块读取 TXT

## 开发

### 环境

- Node.js 24+ 与 pnpm
- Rust stable（MSVC 工具链）
- Visual Studio 2022 Build Tools（MSVC + Windows SDK）
- WebView2 Runtime

### 常用命令

```bash
pnpm install
pnpm tauri dev        # 开发模式
pnpm check            # typecheck + lint + format + test + build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

### 构建 Windows 安装包

```bash
pnpm tauri build
```

产物：`src-tauri/target/release/bundle/nsis/AniShelf_<version>_x64-setup.exe`

### 指定数据目录

便携或测试场景可设置绝对路径环境变量：

```powershell
$env:ANISHELF_DATA_DIR = "D:\AniShelfData"
```

未设置时使用 `%LOCALAPPDATA%\com.anishelf.desktop`。

## 项目结构

```text
src/                      前端（React）
  app/                    路由、导航、Provider
  components/             通用 UI 与布局
  features/               anime / books / reader / notes / search / favorites / dashboard / settings
  stores/                 Zustand UI 状态
src-tauri/                后端（Rust + Tauri）
  src/commands/           Tauri command
  src/services/           业务逻辑
  src/repositories/       数据访问
  src/models/             数据模型
  migrations/             SQLx migration
  icons/                  应用图标
docs/                     验收报告、数据库说明、安全审计、发布说明
AniShelf_ClaudeCode_需求文件包/   V1.0 需求与设计文档
```

## 测试与质量

- Rust：90+ 单元测试（数据库、番剧、书籍、阅读器、笔记、搜索、备份/恢复、封面校验）
- 前端：Vitest + Testing Library；`pnpm check` 覆盖 typecheck、lint、format、test、build
- 安全：最小 Tauri 权限、CSP、Zip Slip 防护、路径与文件类型校验；详见 [docs/security-audit.md](docs/security-audit.md)
- 发布说明：[docs/release-notes-v1.0.md](docs/release-notes-v1.0.md)

## 已知限制

- Markdown 书籍阅读器与编辑器未在 V1.0 提供；`.md` 导入记录保留但不可阅读。
- 批注功能未实现，仅提供书签；笔记暂不能关联批注。
- PDF 仅画布阅读，不支持文本选取/高亮；超大 PDF 首次生成封面可能较慢。
- TXT 进度为按字节偏移的近似值，恢复时向前对齐到行首。
- 搜索每类最多返回 20 条；1–2 字查询使用 LIKE 回退，数据量很大时性能低于全文索引。
- 阅读时长只统计正常结束的阅读会话；强制退出可能留下未结束会话。
- 应用外壳固定深色主题；阅读器内可切换深色/浅色/羊皮纸。
- 安装包未签名，暂无自动更新。
- Bangumi 访问在中国大陆可能需要网络代理，失败不影响本地功能。

## 文档

- [需求与设计文档](AniShelf_ClaudeCode_需求文件包/)
- [数据库结构](docs/database/schema.md)
- [安全审计](docs/security-audit.md)
- [V1.0.1 发布说明](docs/release-notes-v1.0.1.md)
- [V1.0.0 发布说明](docs/release-notes-v1.0.md)
- [阶段验收报告](docs/acceptance/)

## 致谢

感谢 [Tauri](https://tauri.app/)、[React](https://react.dev/)、[shadcn/ui](https://ui.shadcn.com/)、[epub.js](https://github.com/futurepress/epub.js)、[PDF.js](https://mozilla.github.io/pdf.js/)、[SQLx](https://github.com/launchbadge/sqlx) 与 [Bangumi](https://bgm.tv/) 提供的开放接口。

## 许可

本项目尚未指定开源许可证。在添加许可证前，请勿默认将其视为可自由分发或商用；如需使用代码，请先与作者联系。
