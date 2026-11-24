# AniShelf 技术设计与实现约束

## 1. 总体架构

采用 Tauri 2 桌面架构：

- React 前端负责界面、交互和阅读器呈现；
- Rust 后端负责文件系统访问、数据库访问、导入校验、系统能力和业务命令；
- SQLite 负责本地结构化数据；
- Bangumi API 作为可选外部数据源。

建议目录：

```text
AniShelf/
├─ src/
│  ├─ app/
│  ├─ components/
│  ├─ features/
│  │  ├─ anime/
│  │  ├─ books/
│  │  ├─ reader/
│  │  ├─ notes/
│  │  ├─ search/
│  │  ├─ dashboard/
│  │  └─ settings/
│  ├─ stores/
│  ├─ lib/
│  ├─ hooks/
│  └─ types/
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/
│  │  ├─ services/
│  │  ├─ repositories/
│  │  ├─ models/
│  │  ├─ integrations/
│  │  └─ lib.rs
│  ├─ migrations/
│  └─ tauri.conf.json
├─ tests/
├─ docs/
└─ package.json
```

前端按 feature 组织，避免跨领域组件互相耦合。Tauri commands 只负责参数接收、校验和调用 service，不在 command 中堆积复杂业务逻辑。

## 2. 数据库实体建议

以下为逻辑实体清单。具体字段、类型、约束、索引和 migration 应由 Claude Code 在实现前进一步细化并输出 ER 图/数据字典。

### 番剧域
- `anime`：番剧主表，包含本地 ID、Bangumi ID、标题、别名、简介、封面、年份、季度、集数、外部元数据更新时间、用户状态、用户评分、个人评论等。
- `anime_episodes`：单集信息。
- `watch_records`：观看历史，允许补录和多次观看。
- `anime_metadata_sync_logs`：Bangumi 同步日志。

### 书籍与阅读域
- `books`：书籍元数据、格式、原文件路径、文件大小、导入时间、封面路径等。
- `reading_progress`：每本书当前阅读位置和格式专属定位信息。
- `reading_sessions`：阅读会话历史。
- `bookmarks`：书签。
- `annotations`：批注。
- `reader_settings`：阅读器偏好设置。

### 笔记域
- `notes`：笔记标题、正文、创建/更新时间。
- `note_links`：笔记之间的链接。
- `note_relations`：笔记与番剧/书籍/批注等实体的关联。
- `tags`：标签。
- `entity_tags`：标签与各实体的关联。
- `annotation_notes`：批注与笔记的关联（应在 notes 和 annotations 创建之后创建）。

### 通用域
- `favorites`：收藏关系（若采用多态结构，Service 层校验实体存在性）。
- `attachments`：笔记附件/应用管理的资源。
- `activity_logs`：必要的用户活动记录。
- FTS 虚拟表：番剧、书籍、笔记等可搜索内容。

## 3. Migration 规划

建议按以下批次拆分，实际可合并，但必须保持依赖顺序：

1. `0001_initial.sql`：基础设置/通用结构；
2. `0002_anime.sql`：番剧、集数、观看记录；
3. `0003_books.sql`：书籍、阅读进度、阅读会话；
4. `0004_reader_annotations.sql`：书签、批注、阅读设置；
5. `0005_notes.sql`：笔记、标签、笔记链接；
6. `0006_relations.sql`：跨实体关联、附件、收藏；
7. `0007_activity_logs.sql`：活动记录、同步日志；
8. `0008_search.sql`：FTS 与同步触发器。

要求：
- 明确所有主键策略和时间字段格式；
- 对关键外键设置 `ON DELETE` 行为；
- 所有多态关系由 service 层校验；
- FTS 外部内容表必须正确同步 rowid；
- 为 FTS 的 INSERT/UPDATE/DELETE 建立触发器；
- 中文检索必须实测；
- migration 必须在空库和已有库升级场景下测试；
- 不允许只创建 FTS 虚拟表却不维护索引内容。

SQLite 运行配置：
- `PRAGMA foreign_keys = ON;`
- 合理配置 `journal_mode = WAL`；
- 配置 `busy_timeout`；
- 数据库升级前备份；
- migration 记录版本与执行结果。

## 4. Rust 后端分层

建议：
- `commands/`：Tauri command 接口；
- `services/`：业务流程与事务编排；
- `repositories/`：SQL 查询与持久化；
- `models/`：领域模型/DTO；
- `integrations/bangumi/`：Bangumi API 客户端、DTO、映射；
- `file_system/` 或对应 service：文件导入、元数据提取、路径校验。

所有文件路径操作必须在 Rust 后端校验。前端不得拥有任意路径读写能力。

## 5. 阅读器 Adapter

统一接口概念：

```ts
interface ReaderAdapter {
  open(source: ReaderSource): Promise<void>;
  close(): Promise<void>;
  getLocation(): Promise<ReaderLocation>;
  goTo(location: ReaderLocation): Promise<void>;
  next(): Promise<void>;
  previous(): Promise<void>;
  setPreferences(preferences: ReaderPreferences): Promise<void>;
}
```

具体 Adapter：
- `EpubReaderAdapter`：epub.js；
- `PdfReaderAdapter`：PDF.js；
- `TxtReaderAdapter`：Rust 分块读取 + 前端渲染；
- `MarkdownReaderAdapter`：Markdown parser + 编辑器/预览。

统一位置模型必须允许不同格式保存各自定位信息，例如 EPUB 的 CFI、PDF 页码、TXT 字符偏移、Markdown 锚点/滚动位置。

## 6. Bangumi API

- API 请求封装在独立 integration 模块；
- 网络失败、限流、无结果、解析失败均有明确错误状态；
- 不将外部 API 作为番剧管理的唯一入口；
- 保存外部 ID，允许后续重新匹配；
- 将外部元数据字段和用户字段分离；
- 同步前展示必要的冲突处理策略；
- 不在日志中输出敏感 Token。

## 7. 文件导入与安全

- 使用 Tauri 系统文件选择器；
- 限定支持格式；
- 校验扩展名和文件实际内容；
- 处理重复导入；
- 文件路径失效时支持重新定位；
- 原始文件只读；
- 不自动扫描磁盘；
- EPUB/ZIP 解包时防 Zip Slip；
- 不执行导入内容中的脚本；
- 对外链采取用户主动打开策略；
- 大文件分块处理；
- 对异常编码和损坏文件给出错误提示。

## 8. 数据目录

使用 Tauri 推荐的应用数据目录 API 获取路径，不硬编码用户目录。目录建议：

```text
AniShelf/
├─ database/
│  └─ anishelf.sqlite
├─ covers/
├─ attachments/
├─ cache/
├─ backups/
└─ logs/
```

用户原始书籍默认留在原位置。若提供“复制到应用资料库”功能，必须由用户主动选择。

## 9. UI 与交互

- 默认深色主题；
- 统一设计 Token、间距、圆角、阴影、字体和状态色；
- 页面必须包含 loading、empty、error 状态；
- 表格/列表需要搜索、筛选和排序；
- 删除操作提供确认；
- 长任务提供进度反馈；
- 键盘操作和基本可访问性不可忽略；
- 不要把所有页面做成同一种 CRUD 表格，应围绕使用场景设计。

## 10. 测试策略

- Rust：service/repository 单元测试；
- 前端：组件与状态逻辑测试；
- 数据库：migration、外键、索引、FTS 同步测试；
- 集成：导入、阅读进度恢复、观看记录、笔记关联；
- 手动：断网、文件丢失、大文件、损坏文件、备份恢复。
