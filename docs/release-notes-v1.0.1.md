# AniShelf V1.0.1 发布说明

- 版本：1.0.1
- 日期：2026-11-24
- 平台：Windows 10/11 x64

## 1. 本次更新

### 新增

- **鼠标滚轮翻页（EPUB / PDF）**
  - EPUB：滚轮向上/向下翻一页；累计滚动量与冷却时间避免触控板一次手势连翻多页。
  - PDF：适合页面时直接翻页；放大后先滚动页面，滚动到顶部/底部后继续滚动才翻页，翻页后回到页面顶部。
  - TXT 保持连续滚动，不改变原有阅读体验。

### 修复

- **EPUB 目录无法跳转到其他章节**
  - 目录 href 按 nav/NCX 文档位置解析为 spine 路径。
  - 目录面板关闭触发的 epub.js 重新布局不再覆盖跳转目标。
- **EPUB 目录/书签跳转后的页数**
  - 跳转后等待布局稳定再读取当前位置；无效页数不再显示为 `0/0`。
- **PDF.js 静态资源路径**
  - wasm、cmaps、standard_fonts、iccs 之前被嵌套复制到 `pdfjs/*/node_modules/...`，导致部分 PDF 的 ICC/图像解码警告；现在复制到正确路径，改善特殊 PDF 的渲染兼容性。

## 2. 验证结果

- Rust：90 个测试通过，`cargo fmt`、`cargo clippy -D warnings` 通过。
- 前端：`pnpm typecheck`、`lint`、`format:check`、`test`、`build` 通过。
- 用户手工验证：EPUB 目录跳转、章节页数、滚轮翻页正常；PDF 滚轮翻页正常。
- 安装包：`pnpm tauri build` 成功生成 NSIS 安装包。

## 3. 构建产物

- 安装包：`AniShelf_1.0.1_x64-setup.exe`
- SHA-256：`（构建后补充）`
- 便携可执行文件：`src-tauri\target\release\anishelf.exe`

安装包未做代码签名；正式传播前建议使用自己的代码签名证书重新构建，或在发布页提供哈希供校验。

## 4. 升级说明

- 直接运行新安装包覆盖安装即可，应用数据与设置保留。
- 数据库结构升级前会自动备份到应用 `backups/` 目录。
- 原始书籍文件不会被修改或删除；数据库备份不包含原始书籍文件。

## 5. 已知限制

与 v1.0.0 相同，详见 [README](../README.md#已知限制) 与 [安全审计](security-audit.md)。
