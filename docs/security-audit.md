# AniShelf V1.0 安全审计

- 日期：2026-11-24
- 版本：1.0.1
- 范围：Tauri command 权限、文件路径与导入、EPUB/ZIP、外链与脚本、SQL、备份恢复、日志与凭据
- 方法：代码审查 + 既有自动化测试 + 手工构建验证

## 结论

未发现高危问题。应用保持最小 Tauri 权限集合，不提供任意文件系统/SQL/HTTP 前端接口；导入、阅读、备份恢复路径均有服务端校验。以下是控制项与残留风险。

## 1. 权限与运行时

| 控制项         | 状态 | 说明                                                                                              |
| -------------- | ---- | ------------------------------------------------------------------------------------------------- |
| Tauri 插件权限 | 通过 | capabilities 仅包含 `core:default`、`dialog:allow-open`、`dialog:allow-save`；无 fs/http/sql 插件 |
| 资产协议范围   | 通过 | asset 协议仅暴露 `$APPLOCALDATA/covers/**/*`                                                      |
| CSP            | 通过 | 生产 CSP 禁止远程脚本/对象/框架；`script-src 'self' 'wasm-unsafe-eval'`；无 `unsafe-eval`         |
| 单实例         | 通过 | 使用 `tauri-plugin-single-instance`，避免两个进程同时写数据库或恢复                               |
| 远程内容       | 通过 | 书籍内容仅本地读取；不自动加载外部脚本或媒体                                                      |

## 2. Tauri command 与参数校验

- 所有命令通过书籍/番剧/笔记 ID 查询数据库，不信任前端提供的实体内容。
- 导入：`canonicalize` 后必须是文件、大小 1 字节–1GB、扩展名白名单（EPUB/PDF/TXT/Markdown），并读取文件头校验真实类型；SHA-256 去重。
- 重新定位：同样执行规范化、文件类型/大小与哈希校验。
- 阅读字节：仅 EPUB/PDF，分别限制 200MB/300MB；TXT 分块偏移与单次大小有上下限；位置 JSON 与阅读设置 JSON 有长度上限。
- 搜索：关键词最长 200 字符，1–2 字符使用转义 LIKE，≥3 字使用 FTS5 trigram。
- 备份/恢复：备份文件校验扩展名、大小、SQLite 文件头、`PRAGMA integrity_check`、`_sqlx_migrations` 存在性与 schema 版本上限；恢复采用暂存 + 重启应用。
- 生成封面：仅接受 `data:image/png;base64,`，校验 PNG 头、IHDR 尺寸（200–2000 × 200–3000）与 2MB 上限；只写入应用 `covers/`，已有封面不覆盖。

## 3. EPUB / ZIP 安全

- 不将 EPUB 内容解包到用户目录；封面字节仅在通过大小限制后写入应用 `covers/`，文件名由哈希生成。
- `resolve_archive_path` 拒绝绝对路径、带协议/盘符的 href 与越界 `..`；单元测试 `archive_paths_cannot_escape_the_root` 覆盖。
- 元数据/容器/OPF/封面读取均有单条目大小上限。
- 书籍原文只读；应用不会修改或删除原始文件。

## 4. 脚本与内容安全

- 前端无 `dangerouslySetInnerHTML`。
- EPUB 使用 epub.js 渲染，`allowScriptedContent: false`；PDF 以画布渲染，不执行 PDF JavaScript。
- TXT 以 React 文本节点渲染；生成的 TXT/PDF 封面只在本地 Canvas 绘制并保存为 PNG。
- 外部链接仅允许 http/https，经 Rust 端 `validate_external_url` 校验后交给系统浏览器；书籍内链接需用户点击，不自动打开。

## 5. SQL 与文件写入

- 查询使用参数绑定；动态排序/筛选通过白名单与 QueryBuilder 绑定拼接。
- `VACUUM INTO` 仅用于生成的备份路径，路径来自应用生成或用户经保存对话框选择；单引号转义。
- 数据库启用 `foreign_keys`、WAL、`busy_timeout`；迁移前自动备份。
- 备份目标路径拒绝覆盖正在使用的数据库文件；覆盖已有备份时先改名保留旧文件并在失败时回滚。

## 6. 日志与凭据

- `tauri-plugin-log` 写入应用 `logs/anishelf.log`，记录启动、迁移、备份/恢复、封面生成与关键错误。
- Bangumi Token 仅存于本机数据库；代码审查未发现 Token 写入日志、控制台或 Git 的路径。
- 日志可能包含本机文件路径，属于本地诊断信息，不会上传。

## 7. 已知残留风险

1. **安装包未签名**：Windows SmartScreen 可能提示未知发布者；正式分发建议使用代码签名证书。
2. **静态数据未加密**：SQLite、书籍与封面以当前 Windows 用户权限存储，依赖操作系统账户隔离；未实现数据库加密。
3. **保存对话框与后端确认边界**：另存为会覆盖用户确认选择的 `.sqlite`/`.db` 文件；后端无法证明确认来自系统对话框，依赖严格 CSP 与本地前端。
4. **复杂文件解析**：PDF.js/epub.js 处理不可信文件，超大或恶意构造文件可能造成内存/CPU 压力；已设置文件大小上限，但无法完全避免解析型拒绝服务。
5. **无自动更新**：V1.0 不包含签名更新通道；升级需手动安装新版本，数据库迁移前会自动备份。
6. **干净环境验证**：本次用全新的临时 `ANISHELF_DATA_DIR` 验证首次启动与迁移；未在全新 Windows 虚拟机或不同 Windows 版本上执行。
7. **依赖漏洞扫描**：本次未运行 `cargo audit`/`pnpm audit` 的在线漏洞库比对，建议发布前在联网环境补充。
8. **数据目录环境变量**：`ANISHELF_DATA_DIR` 可覆盖应用数据目录（仅接受绝对路径），便于便携与测试；本机进程环境可控制该变量，普通用户范围内不构成额外提权。

## 8. 审计证据

- `cargo test`：90 个测试通过，含 Zip Slip、导入校验、备份恢复、封面校验。
- `cargo clippy --all-targets -- -D warnings`：通过。
- `pnpm typecheck / lint / format:check / test / build`：通过。
- `pnpm tauri build`：NSIS 安装包构建结果见发布说明。
