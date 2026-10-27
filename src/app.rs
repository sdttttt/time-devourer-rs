use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetCursorPos, GetWindowLongPtrW, GetWindowRect,
    IDC_SIZEALL, KillTimer, LWA_ALPHA, LoadCursorW, PostQuitMessage, SWP_NOACTIVATE, SWP_NOSIZE,
    SWP_NOZORDER, SetCursor, SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW, SetWindowPos,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_NCDESTROY, WM_PAINT, WM_RBUTTONUP, WM_TIMER,
};

use crate::consts::{
    ANIMATION_INTERVAL_MS, ANIMATION_TIMER_ID, DATE_TIMER_ID, MENU_EXIT_ID, WM_TRAY_ICON,
};
use crate::paint;
use crate::schedule::Schedule;
use crate::time::ClockTime;
use crate::tray::{self, TrayIcon};

#[derive(Clone, Copy)]
struct DragState {
    cursor: POINT,
    window: RECT,
}

pub struct AppState {
    schedule: Schedule,
    drag: Option<DragState>,
    tray: Option<TrayIcon>,
    #[cfg(debug_assertions)]
    force_show: bool,
}

impl AppState {
    fn new(hwnd: HWND) -> Self {
        let hinstance = crate::window::instance().ok();
        Self {
            schedule: Schedule::new(ClockTime::now()),
            drag: None,
            tray: hinstance.and_then(|instance| TrayIcon::add(hwnd, instance)),
            #[cfg(debug_assertions)]
            force_show: std::env::var_os("TD_FORCE_SHOW").is_some(),
        }
    }
}

/// 取出窗口状态，借用在闭包结束处立即归还。
///
/// 安全性约定：状态指针只由 `WM_CREATE` 写入、`WM_NCDESTROY` 清除，
/// 且窗口过程只在创建它的 UI 线程上执行；空指针时返回 `None`。
fn with_state<R>(hwnd: HWND, f: impl FnOnce(&mut AppState) -> R) -> Option<R> {
    // SAFETY: 见函数文档；指针非空时指向有效的 Box<AppState>。
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState };
    if pointer.is_null() {
        return None;
    }
    // SAFETY: 同上；闭包执行期间 Box 不会被释放。
    Some(unsafe { f(&mut *pointer) })
}

pub unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            // SAFETY: 新分配的 Box 交给窗口长期持有。
            unsafe {
                let state = Box::new(AppState::new(hwnd));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
            }
            LRESULT(0)
        }

        WM_NCDESTROY => {
            // SAFETY: 取回并释放 WM_CREATE 分配的 Box；随后清空指针。
            unsafe {
                let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
                if !pointer.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    drop(Box::from_raw(pointer));
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }

        WM_TIMER if wparam.0 == DATE_TIMER_ID => {
            on_date_timer(hwnd);
            LRESULT(0)
        }

        WM_TIMER if wparam.0 == ANIMATION_TIMER_ID => {
            on_animation_timer(hwnd);
            LRESULT(0)
        }

        WM_PAINT => {
            paint::paint(hwnd);
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            on_lbutton_down(hwnd);
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            on_mouse_move(hwnd);
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            on_lbutton_up(hwnd);
            LRESULT(0)
        }

        WM_TRAY_ICON => {
            on_tray(hwnd, lparam);
            LRESULT(0)
        }

        WM_COMMAND => {
            on_command(hwnd, wparam);
            LRESULT(0)
        }

        WM_CLOSE => {
            // SAFETY: hwnd 有效。
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            on_destroy(hwnd);
            LRESULT(0)
        }

        // SAFETY: 未处理消息交还系统默认过程。
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn on_date_timer(hwnd: HWND) {
    let started = with_state(hwnd, |state| {
        #[cfg(debug_assertions)]
        let force = state.force_show;
        #[cfg(not(debug_assertions))]
        let force = false;

        state.schedule.update(ClockTime::now(), force)
    })
    .unwrap_or(false);

    if started {
        // SAFETY: hwnd 有效；同一 ID 的定时器会被更新。
        unsafe {
            SetTimer(Some(hwnd), ANIMATION_TIMER_ID, ANIMATION_INTERVAL_MS, None);
        }
    }

    // SAFETY: hwnd 有效。
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, true);
    }
}

fn on_animation_timer(hwnd: HWND) {
    let tick = with_state(hwnd, |state| state.schedule.animation_tick());

    if let Some(tick) = tick {
        // SAFETY: hwnd 有效。
        unsafe {
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), tick.alpha, LWA_ALPHA);
            if !tick.animating {
                let _ = KillTimer(Some(hwnd), ANIMATION_TIMER_ID);
            }
        }
    }
}

fn on_lbutton_down(hwnd: HWND) {
    let mut cursor = POINT::default();
    let mut window = RECT::default();

    // SAFETY: 输出参数均为有效可写指针。
    unsafe {
        let _ = GetCursorPos(&mut cursor);
        let _ = GetWindowRect(hwnd, &mut window);
    }

    let _ = with_state(hwnd, |state| {
        state.drag = Some(DragState { cursor, window });
    });

    // SAFETY: hwnd 有效。
    unsafe {
        SetCapture(hwnd);
        if let Ok(cursor) = LoadCursorW(None, IDC_SIZEALL) {
            SetCursor(Some(cursor));
        }
    }
}

fn on_mouse_move(hwnd: HWND) {
    let drag = with_state(hwnd, |state| state.drag).flatten();
    let Some(drag) = drag else {
        return;
    };

    let mut cursor = POINT::default();
    // SAFETY: cursor 是有效可写指针。
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        return;
    }

    let x = drag.window.left + (cursor.x - drag.cursor.x);
    let y = drag.window.top + (cursor.y - drag.cursor.y);

    // SAFETY: hwnd 有效。
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

fn on_lbutton_up(hwnd: HWND) {
    let _ = with_state(hwnd, |state| {
        state.drag = None;
    });

    // SAFETY: 与 SetCapture 配对。
    unsafe {
        let _ = ReleaseCapture();
    }
}

fn on_tray(hwnd: HWND, lparam: LPARAM) {
    match lparam.0 as u32 {
        WM_RBUTTONUP => {
            // 注意：此处不能持有 &mut AppState。
            if tray::show_menu(hwnd) == MENU_EXIT_ID {
                // SAFETY: 菜单模态循环已经结束。
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
            }
        }

        WM_LBUTTONUP => {
            let _ = with_state(hwnd, |state| state.schedule.flash());

            // SAFETY: hwnd 有效。
            unsafe {
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
                SetTimer(Some(hwnd), ANIMATION_TIMER_ID, ANIMATION_INTERVAL_MS, None);
            }
        }

        _ => {}
    }
}

fn on_command(hwnd: HWND, wparam: WPARAM) {
    let id = wparam.0 & 0xFFFF;
    if id == MENU_EXIT_ID {
        // SAFETY: hwnd 有效。
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }
}

fn on_destroy(hwnd: HWND) {
    // 窗口句柄此时仍有效，先显式移除托盘图标；Drop 作为兜底。
    let _ = with_state(hwnd, |state| {
        if let Some(tray) = &mut state.tray {
            tray.remove();
        }
    });

    // SAFETY: hwnd 有效。
    unsafe {
        let _ = KillTimer(Some(hwnd), DATE_TIMER_ID);
        let _ = KillTimer(Some(hwnd), ANIMATION_TIMER_ID);
        PostQuitMessage(0);
    }
}
