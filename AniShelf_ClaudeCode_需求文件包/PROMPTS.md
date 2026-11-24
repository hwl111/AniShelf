# Claude Code 分阶段执行指令

使用方式：每次只复制当前阶段的 Prompt。Claude Code 应先阅读根目录 `CLAUDE.md`、`PRODUCT_REQUIREMENTS.md`、`TECHNICAL_DESIGN.md` 和 `DEVELOPMENT_PLAN.md`。

---

## 阶段 0：检查环境并初始化项目

```text
你现在是 AniShelf 项目的资深 Windows 桌面应用工程师。请先完整阅读项目根目录中的 CLAUDE.md、PRODUCT_REQUIREMENTS.md、TECHNICAL_DESIGN.md、DEVELOPMENT_PLAN.md 和 ENVIRONMENT_SETUP.md。

本阶段只做环境检查和项目初始化，不要直接开始实现业务功能。

任务：
1. 检查 Windows 版本、架构、Git、Node.js、npm、pnpm、Rust、Cargo、Visual Studio C++ Build Tools、Windows SDK、WebView2 Runtime。
2. 输出每项的检测结果、版本、是否满足 Tauri 2 要求。
3. 对缺失组件，提供官方安装链接、安装用途、是否需要管理员权限；先等待我确认，不要擅自安装或修改系统环境变量。
4. 如果环境满足要求，在当前项目目录初始化 Tauri 2 + React + TypeScript + Vite。
5. 配置 pnpm、TypeScript strict、ESLint、Prettier、Tailwind CSS、shadcn/ui。
6. 建立 feature-based 目录结构和基础路由。
7. 实现深色主题主布局，暂时不做业务页面。
8. 运行 pnpm install、前端 typecheck/lint/build、cargo check；若环境允许，启动 pnpm tauri dev 验证窗口。
9. 建立合理 .gitignore，禁止提交 node_modules、target、构建产物、数据库运行文件、用户书籍和密钥。

安全要求：
- 不删除现有文件；
- 不覆盖已有项目配置，除非先展示差异并得到确认；
- 不运行 git reset --hard、git clean 等破坏性命令；
- 不自动执行管理员命令。

完成后汇报：环境检查表、项目结构、安装/初始化命令、测试结果、未解决问题和下一阶段建议。
```

## 阶段 1：项目骨架与基础 UI

```text
请阅读项目需求和开发计划，完成阶段 1：项目骨架与基础 UI。

要求：
1. 检查现有项目，不要重复初始化。
2. 完善 Tauri 2 + React + TypeScript + Vite 工程。
3. 配置 Tailwind CSS、shadcn/ui、React Router、Zustand、TanStack Query。
4. 建立 app、components、features、stores、hooks、lib、types 等目录。
5. 建立 Dashboard、Anime、Bookshelf、Reader、Notes、Search、Settings 页面路由。
6. 实现统一深色主题、侧边栏、顶部区域、页面容器、面包屑和通知。
7. 所有页面先提供真实的布局骨架、空状态和加载状态，不要用大量假数据伪装已完成业务。
8. 配置错误边界、统一按钮/输入框/对话框等基础组件。
9. 运行 typecheck、lint、build 和 cargo check。

不要提前实现数据库和业务逻辑。完成后列出变更文件与验收结果。
```

## 阶段 2：数据库与 Repository/Service

```text
请完成阶段 2：SQLite 数据层。

先根据 TECHNICAL_DESIGN.md 输出 ER 图（Mermaid）和数据字典，说明每张表的字段、主键、外键、唯一约束、索引和删除策略。确认依赖关系后再创建 migration。

实现要求：
1. SQLite 连接、连接生命周期和错误处理；
2. foreign_keys、WAL、busy_timeout；
3. migration runner 和版本记录；
4. anime、anime_episodes、watch_records、books、reading_progress、reading_sessions、bookmarks、annotations、notes、note_links、note_relations、tags、entity_tags、favorites、attachments、activity_logs、同步日志等表；
5. `annotation_notes` 必须在 notes 和 annotations 表创建后创建；
6. 多态关联表的实体存在性必须由 Service 层校验；
7. FTS 表必须具备 INSERT/UPDATE/DELETE 同步触发器；
8. 中文搜索要实际验证，不要默认 SQLite tokenizer 能满足中文检索；
9. migration 前备份数据库；
10. 为关键 repository/service 编写测试。

必须测试：空库完整迁移、重复启动、外键约束、FTS 索引同步、异常回滚。不要直接清空任何已有数据库。
```

## 阶段 3：番剧管理与 Bangumi

```text
请完成阶段 3：番剧管理。

功能：
1. 番剧列表、详情、手动添加、编辑和删除；
2. 观看状态：想看、在看、已看、搁置、弃番；
3. 评分、个人评论、标签、收藏；
4. 集数管理、单集已看状态、观看时间和历史记录；
5. Bangumi 搜索、详情获取、导入和手动刷新；
6. 断网时手动管理必须正常使用；
7. 外部元数据与用户个人数据分开保存。

特别约束：
Bangumi 同步不能覆盖用户评分、观看状态、观看历史、个人标签、评论和笔记。同步冲突要有明确策略和日志。API 失败不能破坏本地数据。

请提供可操作 UI、空/加载/错误状态、service/repository 测试和完整验收步骤。
```

## 阶段 4：书籍资料库与导入

```text
请完成阶段 4：书籍资料库与文件导入。

支持格式：EPUB、PDF、TXT、Markdown。

实现：
1. 使用 Tauri 系统文件选择器；
2. 文件类型识别和导入校验；
3. 书名、作者、格式、文件大小、文件路径、封面、导入时间等元数据；
4. 重复导入检测；
5. 书籍列表、详情、标签、收藏；
6. 文件路径失效检测与重新定位；
7. 原始文件默认只读；
8. 用户可选择是否复制到应用资料库，不得默认擅自复制或移动；
9. 封面和应用附件存放在 Tauri 应用数据目录；
10. EPUB/ZIP 解包必须防 Zip Slip。

不要自动扫描磁盘，不执行导入文件中的脚本，不自动打开外部链接。对损坏文件、无权限文件和不支持格式显示清晰错误。完成后测试四种格式的导入。
```

## 阶段 5A：EPUB 阅读器

```text
请实现 EPUB Reader Adapter，使用 epub.js。

功能：打开 EPUB、目录导航、翻页/滚动、字体大小、行距、阅读主题、保存 CFI 阅读位置、恢复上次位置、书签。批注能力在评估 epub.js 能力后实现，不要伪造支持。

要求：阅读进度持久化；关闭/重启后恢复；异常 EPUB 不导致应用崩溃；外部链接由用户主动点击后再打开。提供测试和手动验收步骤。
```

## 阶段 5B：PDF 阅读器

```text
请实现 PDF Reader Adapter，使用 PDF.js。

功能：打开 PDF、页码跳转、上一页/下一页、缩放、适合宽度/页面、保存页码和缩放状态、恢复上次位置、书签。文本批注仅在技术可行并经过测试后交付。

注意大 PDF 的渲染性能、错误提示和外部链接安全。不要把 PDF 文件内容上传到任何服务。
```

## 阶段 5C：TXT 阅读器

```text
请实现 TXT Reader Adapter。

重点：
1. 编码检测，覆盖 UTF-8、UTF-8 BOM、GBK/GB18030 等常见编码；
2. Rust 后端分块读取；
3. 大文件不能一次性全部读入前端内存；
4. 使用分页或虚拟列表；
5. 保存字符偏移/段落偏移等定位信息；
6. 恢复阅读位置；
7. 支持字体大小、行距、主题。

请使用大文件样本测试内存和滚动表现，并说明编码检测失败时的降级策略。
```

## 阶段 5D：Markdown 阅读与编辑

```text
请实现 Markdown Reader Adapter。

功能：阅读预览、编辑模式、保存、未保存提示、阅读位置恢复、基础 Markdown 语法渲染。原始 Markdown 文件默认只读打开；只有用户进入编辑并明确保存时才写入。保存前处理外部文件被修改的冲突，避免静默覆盖。

不要渲染不可信 HTML 或执行脚本。提供预览、编辑、保存和冲突测试。
```

## 阶段 6：笔记系统

```text
请完成阶段 6：笔记系统。

实现笔记 CRUD、Markdown 编辑/预览、标签、双向链接、反向链接，以及笔记与番剧、书籍、章节、批注的关联。提供关联实体选择器和从作品详情页跳转笔记的入口。

删除笔记或关联实体时，必须提示引用影响并按明确策略处理。不要静默丢失链接。实现笔记全文搜索和相关测试。
```

## 阶段 7：搜索、收藏、统计与 Dashboard

```text
请完成阶段 7。

实现全局搜索，覆盖番剧标题/别名/简介、书名/作者、笔记标题/正文和可检索批注。中文搜索必须经过测试并说明 tokenizer 或应用层分词方案。

实现收藏、首页真实数据汇总、观看/阅读统计和活动时间线。提供筛选、排序、分页、空状态。禁止使用硬编码数据冒充真实统计。
```

## 阶段 8：备份、稳定性与 Windows 发布

```text
请完成阶段 8：备份恢复、稳定性、安全审计和 Windows 发布。

任务：
1. SQLite 备份和恢复；
2. 恢复前自动备份当前数据库；
3. 恢复失败不破坏现有数据；
4. 书籍原文件位于应用目录之外时，明确提示需要单独备份；
5. 数据库升级前自动备份；
6. 日志、错误反馈和异常处理；
7. 审计路径校验、文件导入、Zip Slip、外链、Tauri command 权限；
8. 完成 Windows 安装包构建；
9. 在干净环境验证启动；
10. 执行核心回归测试。

最终输出：功能完成情况、构建产物位置、测试结果、已知限制、安全检查结果和 V1.0 发布说明草稿。
```
