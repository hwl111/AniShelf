# 阶段 2 验收报告

- 日期：2026-11-24
- 结论：**通过**

## 1. 完成内容

1. **数据库设计**：`docs/database/schema.md`，含 Mermaid ER 图、数据字典、约束/索引、删除与多态清理规则、Bangumi 字段边界、中文检索方案、8 个 migration 批次；经用户确认（评分 1–10、四季英文枚举、单分类 + 标签）。
2. **SQLite 基础设施**：
   - SQLx 0.9.1（sqlite-bundled，含 FTS5）。
   - 应用数据目录：`database/`、`covers/`、`attachments/`、`cache/`、`backups/`、`logs/`。
   - 连接池：`foreign_keys=ON`、WAL、`busy_timeout=5s`、`synchronous=NORMAL`、最大 4 连接。
   - 统一 `AppError` 序列化给前端。
3. **Migration 框架**：SQLx 嵌入式迁移；版本记录；重复启动幂等；存在既有 schema 时先用 `VACUUM INTO` 备份，备份失败不迁移；支持部分迁移后的升级。
4. **Schema（0001–0008）**：20 张实体表 + 4 张 FTS5 虚拟表；CHECK、唯一约束、索引、级联与 SET NULL 策略齐全；`annotation_notes` 在 notes/annotations 之后创建。
5. **中文全文检索**：FTS5 `trigram`；四张外部内容表 + 12 个 INSERT/UPDATE/DELETE 同步触发器 + rebuild；搜索服务与 `search_all` command：≥3 字 MATCH、1–2 字转义 LIKE 降级、limit 20。
6. **设置页**：通过 IPC 展示真实数据库路径、文件大小、journal_mode、外键状态、已应用迁移数量，含加载/错误/刷新状态。

## 2. 提交记录

| 提交      | 说明                                    |
| --------- | --------------------------------------- |
| `8607f64` | docs(db): ER 图与数据字典               |
| `13af3c5` | chore(docs): 表格对齐                   |
| `6f81f74` | feat(db): SQLite 连接池、目录与状态命令 |
| `9fab122` | feat(settings): 真实数据库状态          |
| `0a8a1ba` | feat(db): 迁移 runner 与迁移前备份      |
| `6b770e5` | feat(db): 番剧/书籍/阅读 schema         |
| `1b2691a` | feat(db): 笔记/关联/日志 schema         |
| `abdd7c8` | feat(search): FTS5 中文检索             |
| `a42753c` | test(db): 部分迁移升级与备份            |

## 3. 验收结果

| 检查                                             | 结果                                                        |
| ------------------------------------------------ | ----------------------------------------------------------- |
| `cargo test`                                     | 24 个测试通过                                               |
| `cargo fmt --check` / `cargo clippy -D warnings` | 通过                                                        |
| `pnpm check`（typecheck/lint/format/test/build） | 通过，前端 4 个测试                                         |
| `pnpm tauri build --debug --no-bundle`           | 通过                                                        |
| 空库全量迁移                                     | 通过（测试）                                                |
| 重复启动不重复执行                               | 通过（测试）                                                |
| 部分迁移升级 + 自动备份 + 数据保留               | 通过（测试）                                                |
| 外键、级联、SET NULL、唯一约束、CHECK            | 通过（测试）                                                |
| FTS 新增/更新/删除同步、integrity-check          | 通过（测试）                                                |
| 中文 3/5 字命中、2 字 LIKE 降级、通配符转义      | 通过（测试）                                                |
| 实际应用首次启动                                 | 用户确认：设置页显示真实路径、WAL、外键已启用、**8 个迁移** |

## 4. 已知事项

1. 业务 repository/service（番剧、书籍、笔记 CRUD）尚未实现，按阶段 3–6 推进；当前仅完成数据层、搜索与状态命令。
2. 多态关联的存在性校验与删除清理规则已写入设计文档，但对应 service 实现随各业务阶段落地。
3. `search_all` 命令已可用，搜索页面 UI 在阶段 7 完成。
4. 1–2 字查询走 LIKE 全表扫描，当前限制 20 条；数据量大时需评估分词索引。
5. Bangumi 同步仅完成表结构与日志设计，真实 API 客户端与冲突策略在阶段 3。
6. 备份/恢复 UI 在阶段 8；当前仅有单元测试覆盖的底层备份能力。
7. 实际数据库已创建在应用数据目录（`%LOCALAPPDATA%\com.anishelf.desktop\database\anishelf.sqlite`），未写入演示数据。
8. 临时脚手架目录 `f:\AniShelf_bootstrap_20261124` 仍在，需手动删除。

## 5. 下一步

阶段 3：番剧管理与 Bangumi —— 番剧 CRUD、观看状态/评分/标签/评论、集数与观看记录、Bangumi 搜索/导入/刷新、同步冲突策略、离线可用。
