# time-devourer

Rust + Win32 API（官方 `windows` crate）实现的桌面悬浮时钟（学习项目）。

- 无边框、置顶、圆角、分层透明窗口
- 纯 GDI 手绘七段数码管 `HH:MM:SS`
- 每个整点前后 30 秒淡入显示，其余时间隐藏
- 托盘图标：左键闪现当前时间，右键菜单退出
- 鼠标拖动窗口

![Screenshot](https://github.com/user-attachments/assets/cb252d64-d6b8-45e0-9171-055de8e261a9)

## 构建

环境要求：

- Rust stable，Windows MSVC toolchain（`x86_64-pc-windows-msvc`）
- Visual Studio Build Tools（含 C++ 工具链与 Windows SDK，提供 `rc.exe`）

```bash
cargo build --release
```

产物为 `target/release/time-devourer.exe`。图标资源由 `build.rs` + `embed-resource` 编译 `timer.rc` 嵌入。

## 开发与验证

```bash
cargo test
cargo clippy --all-targets
cargo fmt
```

debug 构建保留控制台，并提供一个仅调试用的环境变量：

- `TD_FORCE_SHOW=1`：强制显示时钟，便于检查渲染、动画与拖动，无需等到整点。

该入口在 release 构建中不会编译。

## 模块结构

| 模块 | 职责 |
|---|---|
| `src/main.rs` | 入口、消息循环、panic hook |
| `src/app.rs` | `AppState`、wndproc、定时器、拖拽、托盘事件 |
| `src/window.rs` | 窗口类注册/创建、分层 alpha、圆角区域 |
| `src/tray.rs` | 托盘图标 RAII、右键菜单（`TPM_RETURNCMD`） |
| `src/paint.rs` | `BeginPaint/EndPaint` 与面板绘制 |
| `src/segments.rs` | 七段数码管位掩码、几何与共享 GDI 调色板 |
| `src/gdi.rs` | GDI 对象 RAII、`SelectObject` 恢复守卫、`rgb()` |
| `src/time.rs` | `GetLocalTime`、`HH:MM:SS` 格式化、整点距离 |
| `src/schedule.rs` | 可见性策略（含启动行为对齐） |
| `src/fade.rs` | 纯 alpha 动画状态机 |
| `src/consts.rs` | 窗口/布局/颜色/ID 常量 |

## 说明与限制

- 时间基于 `GetLocalTime` 每秒重算，不依赖步进计数，睡眠唤醒后不会漂移。
- 目前不做高 DPI、多显示器、开机自启、配置持久化与单实例。
- 这是学习项目，直接在 Win32 消息模型上实现；FFI 调用集中在少数模块，业务逻辑（`time` / `fade` / `schedule` / `segments`）可单元测试。
