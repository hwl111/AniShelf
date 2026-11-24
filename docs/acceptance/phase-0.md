# 阶段 0 验收报告

- 日期：2026-11-24
- 结论：**通过**

## 1. 环境检查

| 项目 | 结果 |
| --- | --- |
| Windows | 11 Pro 10.0.26200 x64 |
| Git | 2.54.0（user.name/user.email 已配置） |
| Node.js | v24.19.0（来自 Python `nodejs_wheel`，见已知事项） |
| npm | 11.17.0，registry 为 npmmirror |
| pnpm | 12.23.0（用户级全局安装，无需管理员） |
| Rust | 1.100.0 stable-x86_64-pc-windows-msvc |
| Visual Studio | 2022 Community + MSVC 14.44 + Windows SDK 10.0.26100 |
| WebView2 Runtime | 158.0.4370.33 |
| 磁盘 | F: 可用 754GB |

## 2. 完成内容

1. pnpm 安装并验证可执行（store: `F:\.pnpm-store`）。
2. Git 基线：初始化 main 分支、`.gitattributes`（强制 LF，含 `*.sql`）、`.gitignore`、根目录 `CLAUDE.md`。
3. Tauri 2 脚手架：在临时目录生成后选择性合并，未在非空根目录使用 `--force`，需求文档完好。
4. 工程配置：
   - TypeScript strict + `@/*` 路径别名（TypeScript 6 已弃用 `baseUrl`，改用无 baseUrl 的 paths）。
   - ESLint 10 flat config（typescript-eslint、react-hooks、react-refresh、Prettier 兼容）+ Prettier + Tailwind 类排序。
   - Tailwind CSS v4 + 深色基础 token + 中文字体回退。
   - shadcn/ui 初始化（radix-nova、CSS 变量、深色 token）+ Button。
   - Tauri 元信息：AniShelf、1280×800（最小 1024×700）、深色原生主题；identifier 改为 `com.anishelf.desktop`。
5. 质量脚本：`typecheck`、`lint`、`format`、`format:check`、`check`。

## 3. 提交记录

| 提交 | 说明 |
| --- | --- |
| `c38e584` | chore(repo): 初始化仓库 |
| `b4f5263` | chore: Tauri 2 + React 19 + TS 6 + Vite 8 基线 |
| `59df1c8` | build(ts): 路径别名与严格配置 |
| `33ea667` | build(lint): ESLint + Prettier + 质量脚本 |
| `33562ed` | build(tailwind): Tailwind v4 与深色 token |
| `7507357` | feat(ui): shadcn/ui 初始化 + Button |
| `ec0aa06` | chore(tauri): 产品元信息与深色窗口 |
| `9b0eacd` | fix(tauri): 修正 bundle identifier |

## 4. 验收结果

| 检查 | 结果 |
| --- | --- |
| `pnpm check`（typecheck → lint → format:check → build） | 通过 |
| `cargo fmt --check` | 通过 |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| `cargo test` | 通过（当前 0 个测试） |
| `pnpm tauri build --debug --no-bundle` | 通过，产物 `src-tauri/target/debug/anishelf.exe` |
| `pnpm tauri dev` 窗口 | 用户目视确认：标题 AniShelf、约 1280×800、深色界面 |

> 阶段 0 仅验证工程壳；窗口内容为 Tauri 官方模板，属预期。AniShelf 实际界面在阶段 1 实现。

## 5. 已知事项

1. Node.js 由 Python `nodejs_wheel` 提供，当前可用；长期建议安装官方 Node 24 LTS，未经确认不卸载现有版本。
2. pnpm 12 自动生成 `pnpm-workspace.yaml` 的发布年龄豁免（typescript-eslint 8.74.0），已纳入版本控制以保证安装可复现。
3. `tauri.conf.json` 的 CSP 目前为 `null`；在阶段 1 引入真实页面/资源前需配置 CSP（含 dev 环境）。
4. shadcn 生成目录 `src/components/ui/**` 单独关闭 fast-refresh 导出规则，业务代码仍保持开启。
5. 临时脚手架目录 `f:\AniShelf_bootstrap_20261124` 仍存在，待用户确认后删除。
6. 数据库、迁移、业务命令均未开始（阶段 2 起）。

## 6. 下一步

阶段 1：项目骨架与基础 UI —— providers、hash 路由（七个页面）、侧边栏 + 顶栏 + 页面容器、深色设计系统、空/加载/错误状态、toast、错误边界、冒烟测试。
