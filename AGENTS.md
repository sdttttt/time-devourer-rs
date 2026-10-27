# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) and other coding agents when working with code in this repository.

## 项目概览

`time-devourer` 是一个纯 Win32 API（官方 `windows` crate，无 GUI 框架）实现的 Windows 桌面悬浮时钟：

- 无边框、置顶、分层、圆角弹窗，GDI 手绘七段数码管 `HH:MM:SS`
- 每个整点前后 30 秒淡入/淡出，其余时间透明隐藏
- 托盘左键闪现当前时间，右键菜单只有“退出”
- 窗口可拖动；release 为 Windows GUI 子系统，debug 保留控制台

旧 C++ 实现已移除；旧代码仍在迁移前的提交 `ac4df1f` 中（本地另打了 tag `cpp-baseline`）。

## 常用命令

```bash
cargo build                 # debug（带控制台）
cargo build --release       # 产物 target/release/time-devourer.exe
cargo test                  # 全部单元测试
cargo test time::           # 按模块过滤
cargo test fade::tests::fades_in_over_51_ticks   # 单个测试
cargo clippy --all-targets -- -D warnings
cargo fmt                   # 格式化
cargo run                   # 运行 debug；从托盘菜单退出
TD_FORCE_SHOW=1 cargo run   # debug 强制显示，免等整点；release 会忽略
```

构建前提：Rust MSVC toolchain + VS Build Tools/Windows SDK（提供 `rc.exe`）。`build.rs` 用 `embed-resource` 编译根目录的 `timer.rc`、`resource.h`、`icon.ico`；这三个文件不要随意移动，`resource.h` 与 `src/consts.rs` 中的 `IDI_ICON=105` 必须保持一致。首次构建会编译 `windows` crate，耗时较长。

## 架构

单窗口、消息驱动，没有 async 运行时或 GUI 框架。

- `src/main.rs`：获取 hInstance、注册窗口类、创建窗口、`GetMessageW` 循环、panic hook。
- `src/window.rs`：注册 `WNDCLASSEXW`；创建 `WS_POPUP` + `WS_EX_LAYERED|TOOLWINDOW|TOPMOST` 窗口，设置圆角区域、初始 alpha=0、位置 (10,10)、1 秒 `DATE_TIMER_ID`，最后显示窗口。
- `src/app.rs`：全部消息分发。`AppState` 在 `WM_CREATE` 以 `Box::into_raw` 存入 `GWLP_USERDATA`，`WM_NCDESTROY` 取回释放；不要改成全局 `static`。
- 两个定时器：`DATE_TIMER_ID` 每秒从 `GetLocalTime` 重算可见性并 `InvalidateRect`；`ANIMATION_TIMER_ID` 每 10ms 推进 alpha（每帧 ±5，约 0.5 秒完成）。
- `src/tray.rs`：托盘图标 RAII（`NIM_ADD` / `NIM_DELETE`）。右键菜单使用 `TPM_RETURNCMD` **同步返回命令 ID**，避免 `TrackPopupMenu` 泵消息时递归派发 `WM_COMMAND`；`WM_COMMAND` 只保留退出防御分支。
- 重入规则：绝不跨 `TrackPopupMenu`、`DestroyWindow`、`MessageBoxW` 持有 `&mut AppState`；`with_state` 只做短借用。

### 纯逻辑（可单元测试）

- `src/time.rs`：`ClockTime`、`format_hms`、`seconds_from_nearest_hour`（正数=距下一整点，负数=距上一整点）。
- `src/schedule.rs`：可见性策略，`SHOW_RADIUS_SEC=30`；`suppress_current_window` 保留旧行为——启动时若已处于整点后 30 秒内，不补显该窗口。修改时间语义时先跑这里和 `time` 的测试。
- `src/fade.rs`：alpha 状态机，`request()` 返回是否需要启动动画定时器。
- `src/segments.rs`：七段位掩码、段几何与共享 `Palette`。

### 渲染与 GDI 所有权

- `src/paint.rs`：`PaintScope` 保证 `BeginPaint/EndPaint` 配对；每次绘制创建一个共享 `Palette`（面板/点亮/未点亮各一组 brush+pen）。
- `src/gdi.rs`：`OwnedGdi` 独占句柄，`Selected<'a>` 守卫在 Drop 时恢复旧对象。声明顺序必须保证守卫先于 GDI 对象析构，否则会删除仍被选入 DC 的对象。不要把 GDI 对象存进 `AppState` 或跨 `WM_PAINT` 缓存。
- `SetWindowRgn` 成功后 region 所有权归系统，只在失败时 `DeleteObject`。

## 行为不变量

- 可见区间：`abs(seconds_from_nearest_hour(now)) <= 30`，即 `[整点-30s, 整点+30s]`。
- 启动抑制：若启动时处于整点后 30 秒内，保持隐藏，直到该窗口过去（与旧 `StepCountDown` 一致）。
- 窗口：320×100 @ (10,10)，面板内缩 10，圆角 10；面板 RGB 220，未点亮段 RGB 200，点亮段黑色。
- 托盘左键：alpha 置 255 再淡出；若正处于整点窗口内，下一秒会被 `Schedule` 拉回显示（有意保留的旧行为）。
- release 中不编译 `TD_FORCE_SHOW` / `force_show`。

## 兼容性与易错点

- `windows` 0.61.3：许多 API 返回 `Result`；`BOOL` 用 `.0` 或 `.as_bool()`。`GetMessageW` 的错误是 `BOOL(-1)`，必须判 `.0 == -1`，不能只看 `as_bool()`。
- 新增 Win32 API 时，需要在 `Cargo.toml` 的 `windows` features 中加入对应模块，否则符号不存在。
- 宽字符串用 `windows::core::w!`；资源 ID 用 `PCWSTR(IDI_ICON as *const u16)`。
- `WM_TRAY_ICON` 采用 legacy 语义：`lParam` 低 16 位是鼠标消息。
- edition 2024 的 `unsafe_op_in_unsafe_fn`：wndproc 内部的 unsafe 调用需要显式 `unsafe {}` 块。
- 项目有意不做高 DPI、多显示器、开机自启、设置窗口、配置持久化、单实例；不要顺手加入这些功能。
- 注释与用户可见文本使用中文，保持现有风格。依赖保持最小化，`windows` 和 `embed-resource` 使用精确版本，`Cargo.lock` 提交在仓库中。
