# 阶段 1 验收报告

- 日期：2026-11-24
- 结论：**通过**

## 1. 完成内容

1. **运行时依赖**：React Router 8.5.2、Zustand 5.1.0、TanStack Query 5.108.4、Sonner。
2. **应用基础设施**：
   - `AppProviders`：QueryClient（桌面端默认值）、TooltipProvider、深色 Toaster。
   - `AppErrorBoundary`：全局渲染错误兜底，提示不影响本地数据并支持重新加载。
   - `invokeCommand`：类型化 IPC 封装，统一错误为 `IpcError` 并记录日志。
   - `useUiStore`：侧边栏折叠状态。
3. **应用壳**：
   - 固定侧边栏：AniShelf 标识、七个导航项（图标 + 中文标签）、激活态、折叠/展开（折叠时 tooltip）。
   - 顶栏：面包屑（AniShelf / 当前页）与本地优先标识。
   - 主内容区独立滚动；`document.title` 随路由更新。
4. **路由与页面**：hash 路由，七个页面 + 404 + 路由错误页；全部为真实布局骨架和空状态，无演示假数据。
5. **UI 基础组件**：Input、Label、Dialog、Card、Separator、Tooltip、Sonner（Sonner 固定深色主题）。
6. **安全基线**：生产 CSP 与 dev CSP；Tauri capability 收紧为 `core:default`；移除模板 greet 命令与演示组件。
7. **测试**：Vitest + jsdom + Testing Library，覆盖首页渲染、侧边栏导航、404 路由、错误边界。

## 2. 提交记录

| 提交      | 说明                                      |
| --------- | ----------------------------------------- |
| `765df04` | chore(docs): 规范化阶段 0 报告格式        |
| `f115e24` | feat(ui): 核心 shadcn 组件                |
| `2c295ce` | feat(app): providers、IPC 封装、UI store  |
| `51a209e` | feat(ui): 页面容器与状态组件              |
| `d14455a` | feat(routing): 七个路由页面，移除模板演示 |
| `8d39f1f` | feat(ui): 侧边栏 + 顶栏应用壳             |
| `76c7d4c` | build(security): CSP 与 capability 收紧   |
| `62e8af0` | test(ui): Vitest 与冒烟测试               |

## 3. 验收结果

| 检查                                                     | 结果                                  |
| -------------------------------------------------------- | ------------------------------------- |
| `pnpm check`（typecheck → lint → format → test → build） | 通过                                  |
| `cargo fmt --check`                                      | 通过                                  |
| `cargo clippy --all-targets -- -D warnings`              | 通过                                  |
| `cargo test`                                             | 通过（当前 0 个 Rust 测试）           |
| `pnpm test`                                              | 4 个测试通过                          |
| `pnpm tauri build --debug --no-bundle`                   | 通过                                  |
| `pnpm tauri dev` 窗口                                    | 用户确认：界面与路由正常              |
| 生产构建 `anishelf.exe`                                  | 用户确认：显示与路由正常，无 CSP 白屏 |

## 4. 已知事项

1. 侧边栏折叠状态尚未持久化，计划随设置/主题一起接入。
2. 搜索、设置页面暂为空状态；阅读器提供“前往书架”占位入口。
3. 前端产物约 479 kB（gzip 153 kB），暂未做路由级代码分割；可在阶段 7/8 优化。
4. 窗口图标仍是 Tauri 模板图标与 `/vite.svg` favicon，阶段 8 统一替换为 AniShelf 资源。
5. 数据库、migration、Rust business commands 尚未开始（阶段 2）。
6. 临时脚手架目录 `f:\AniShelf_bootstrap_20261124` 仍在；`rm -rf` 被内置安全检查拦截，需手动删除。
7. 阶段 1 引入 `@tauri-apps/plugin-opener` 前端包但未使用；后续若确定外链统一走 Rust 端，可移除。

## 5. 下一步

阶段 2：SQLite 与数据访问层 —— 先交付 ER 图与数据字典并请用户确认，再实现连接配置、migration runner、迁移前备份、0001–0008 schema、FTS5 中文检索与测试。
