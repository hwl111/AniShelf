# 阶段 8 验收报告（备份、稳定性与发布）

- 日期：2026-11-24
- 结论：**通过**
- 版本：v1.0.1

## 1. 完成内容

1. **数据库备份**
   - 应用内「立即备份」与「另存为」；`VACUUM INTO` 生成 `.sqlite` 备份与同名 `.json` 元数据（应用版本、schema、时间、大小）。
   - 备份列表显示手动备份、迁移前备份、恢复前备份及大小/版本。
2. **恢复保护流程**
   - 校验扩展名、文件大小、SQLite 文件头、`PRAGMA integrity_check`、`_sqlx_migrations` 与 schema 版本上限。
   - 恢复前自动备份当前数据库；备份文件暂存为 `pending_restore.sqlite`，重启后应用。
   - 启动时再次校验；替换失败回滚；校验失败保留当前数据库；支持取消待恢复文件。
   - 数据库迁移前自动备份机制保留。
3. **日志与稳定性**
   - `tauri-plugin-log` 写入应用 `logs/anishelf.log`，记录启动、迁移、备份/恢复与封面生成；后端错误统一走日志。
   - 引入 `tauri-plugin-single-instance`，避免双开同时写库/恢复冲突。
4. **安全审计**
   - 输出 `docs/security-audit.md`，覆盖权限、CSP、asset 协议、Zip Slip、路径/文件类型、外链、SQL、备份恢复、Token 与日志。
5. **Windows 发布**
   - 版本 1.0.0/1.0.1、应用图标、publisher/描述/分类/版权元信息。
   - NSIS 安装包（当前用户安装），GitHub 公开仓库与 Release。
6. **后续功能与修复**
   - EPUB 内置封面、PDF 第 1 页封面、TXT 首页文字封面；已有封面不覆盖。
   - 统一 `ANISHELF_DATA_DIR` 数据目录覆盖，用于便携与干净环境验证。
   - EPUB/PDF 鼠标滚轮翻页；PDF 放大时边缘翻页。
   - 修复 EPUB 目录跳转（href 解析 + 布局覆盖）与跳转后章节页数。
   - 修复 PDF.js 静态资源被嵌套复制的问题。

## 2. 提交记录

| 提交      | 说明                                                                  |
| --------- | --------------------------------------------------------------------- |
| `f4c9321` | feat: add backup/restore, file logging and generated cover backend    |
| `1e24849` | feat(settings): add backup/restore UI and about card                  |
| `3628f9a` | feat(books): generate covers from PDF first page and TXT first screen |
| `955c9f6` | chore(release): bump to 1.0.0, add app icon and NSIS bundle metadata  |
| `ef1d3a3` | feat(config): allow ANISHELF_DATA_DIR override for clean-room testing |
| `d513fe2` | docs: add README, security audit and v1.0 release notes               |
| `5dfed8e` | feat(reader): add wheel paging and fix EPUB TOC navigation            |
| `954981c` | fix(build): copy PDF.js assets to flat pdfjs directories              |
| `eb07b85` | chore(release): prepare v1.0.1                                        |

## 3. 验收结果

| 检查                                             | 结果                                                                |
| ------------------------------------------------ | ------------------------------------------------------------------- |
| `cargo test`                                     | 90 个测试通过                                                       |
| `cargo fmt --check` / `cargo clippy -D warnings` | 通过                                                                |
| `pnpm check`（typecheck/lint/format/test/build） | 通过，前端 9 个测试（新增滚轮翻页 5 个）                            |
| `pnpm tauri build`                               | 通过，生成 NSIS 安装包                                              |
| 干净数据目录首次启动                             | 发布版 exe + 全新 `ANISHELF_DATA_DIR`：创建空库并应用 9 个迁移      |
| 备份/恢复完整流程                                | 用户实际执行：创建备份 → 暂存恢复 → 重启 → 自动备份当前库并成功恢复 |
| 封面补齐                                         | 用户确认：EPUB 保留内置封面；PDF/TXT 自动生成封面                   |
| EPUB 目录跳转与章节页数                          | 用户确认正常                                                        |
| EPUB/PDF 滚轮翻页                                | 用户确认正常                                                        |
| 既有阅读、笔记、搜索、收藏、首页回归             | 用户确认正常                                                        |
| GitHub 仓库 / Release                            | `hwl111/AniShelf` 公开仓库；v1.0.0 已发布，v1.0.1 发布包            |

## 4. 已知事项

1. 安装包未做代码签名，SmartScreen 可能提示未知发布者；未在全新 Windows 虚拟机安装验证，仅用全新数据目录验证首次启动。
2. 数据库与书籍静态存储未加密，依赖 Windows 用户账户隔离。
3. Markdown 阅读/编辑、批注功能未实现；PDF 不支持文本选取与高亮。
4. 应用外壳固定深色主题；阅读器内可切换深色/浅色/羊皮纸。
5. 安装包不含自动更新；后续升级需手动安装。
6. 阅读时长只统计正常结束的阅读会话。
7. 公开仓库提交历史包含提交者邮箱；如需隐藏可改用 GitHub noreply 并重写历史。
8. 发布前建议补充 `cargo audit` / `pnpm audit` 与代码签名。

## 5. 发布产物

- GitHub 仓库：https://github.com/hwl111/AniShelf
- v1.0.0 Release：https://github.com/hwl111/AniShelf/releases/tag/v1.0.0
- v1.0.1 Release：https://github.com/hwl111/AniShelf/releases/tag/v1.0.1
- 安装包与 SHA-256：见 v1.0.1 Release 资产与 `docs/release-notes-v1.0.1.md`

## 6. 下一步

V1.0 发布完成。后续维护建议：代码签名、依赖漏洞扫描、批注/PDF 文本高亮、可选浅色主题、自动更新与在干净 Windows 虚拟机上的安装回归。
