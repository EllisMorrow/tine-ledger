# Windows 本机测试：只读诊断 Alpha

此包用于确认 Tine 能加载真实 Wasm 插件、命令能收到当前块，以及规范流水格式的校验能运行。**它还没有快捷记账、分类弹层、跨页汇总或图表功能，不能当作完成的账本使用。** 完整版等待官方宿主接口。

## 安装包与隔离图谱

- 使用本地交付工作区 `builds/` 中的诊断安装包。
- 解压后，在 Tine 的设置 → 插件中选择本地安装包，同时选择 `manifest.json` 和 `plugin.wasm`。
- 安装后默认停用；检查包只声明 `commands.register`，再启用。
- 本地交付使用工作区 `sandbox/ledger-smoke-graph/` 合成图谱。源码中的 `tests/fixtures/graph/` 是原始样例，应复制到独立目录后再打开。不要复制真实账目作为测试材料。
- 此流程不需要替换 Tine 程序、手动修改 AppData 或连接外部服务。

## 测试步骤

1. 在命令面板搜索并运行 **Ledger: Show API readiness**。应显示诊断版本及缺失宿主能力的说明。
2. 打开隔离图谱中的 `Ledger smoke test` 页面，先把光标放在那笔普通的 `Lunch` 流水块中。
3. 打开命令面板，运行 **Ledger: Validate focused block (diagnostic alpha)**。
4. 应显示 `Canonical CNY record: expense 28.00`，并说明这只是只读格式校验，未检查引用存在性或全库统计。
5. 依次在 `Own-account transfer` 和 `Refund` 块运行同一命令，检查能通过相应规范格式校验。
6. 在 `Invalid amount` 块运行校验，应显示错误，不应修改任何笔记。
7. 在包含代码示例的块中运行，代码围栏中的属性不能被识别为真实流水。
8. 停用插件后，两个命令消失，测试图谱仍然可读。

若出现错误，只需要反馈 Tine 版本、运行了哪一步、提示文字；界面交互由用户完成。不要把“Wasm 检查通过”理解为“完整账本功能已经通过本机验收”。

## 开发检查

```text
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release
node scripts/test-wasm.cjs
node <Tine checkout>/scripts/tine-plugin.mjs check <this directory> --json
```

本项目锁定官方模板提供的 SDK revision。实验版 manifest 使用 API 0.2 的基础贡献点，避免为没有用到的 0.3 颜色设置提高最低版本。
