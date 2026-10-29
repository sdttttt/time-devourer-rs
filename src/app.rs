use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_NCDESTROY, WM_PAINT, WM_RBUTTONUP, WM_TIMER,
};

use crate::consts::{
    ANIMATION_INTERVAL_MS, ANIMATION_TIMER_ID, DATE_TIMER_ID, MENU_EXIT_ID, WM_TRAY_ICON,
};
use crate::paint;
use crate::schedule::Schedule;
use crate::sys::tray::{TrayIcon, show_menu};
use crate::sys::window;
use crate::time::ClockTime;

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
        let hinstance = window::instance().ok();
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
    let pointer = window::user_data(hwnd).cast::<AppState>();
    if pointer.is_null() {
        return None;
    }
    // SAFETY: 见函数文档；指针非空时指向有效的 Box<AppState>，闭包执行期间不会被释放。
    Some(unsafe { f(&mut *pointer) })
}

// SAFETY: 由系统在窗口所属 UI 线程上调用；未处理消息交还 DefWindowProcW。
pub unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let state = Box::new(AppState::new(hwnd));
            // 裸指针只是存进窗口槽位，所有权在 WM_NCDESTROY 时收回。
            window::set_user_data(hwnd, Box::into_raw(state).cast());
            LRESULT(0)
        }

        WM_NCDESTROY => {
            let pointer = window::user_data(hwnd).cast::<AppState>();
            if !pointer.is_null() {
                window::set_user_data(hwnd, std::ptr::null_mut());
                // SAFETY: 收回 WM_CREATE 分配的 Box；槽位已清空，不会二次释放。
                unsafe {
                    drop(Box::from_raw(pointer));
                }
            }
            window::default_window_proc(hwnd, message, wparam, lparam)
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
            window::destroy_window(hwnd);
            LRESULT(0)
        }

        WM_DESTROY => {
            on_destroy(hwnd);
            LRESULT(0)
        }

        _ => window::default_window_proc(hwnd, message, wparam, lparam),
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
        window::start_timer(hwnd, ANIMATION_TIMER_ID, ANIMATION_INTERVAL_MS);
    }
    window::invalidate(hwnd);
}

fn on_animation_timer(hwnd: HWND) {
    let tick = with_state(hwnd, |state| state.schedule.animation_tick());

    if let Some(tick) = tick {
        window::set_alpha(hwnd, tick.alpha);
        if !tick.animating {
            window::stop_timer(hwnd, ANIMATION_TIMER_ID);
        }
    }
}

fn on_lbutton_down(hwnd: HWND) {
    let cursor = window::cursor_pos().unwrap_or_default();
    let rect = window::window_rect(hwnd).unwrap_or_default();

    let _ = with_state(hwnd, |state| {
        state.drag = Some(DragState {
            cursor,
            window: rect,
        });
    });

    window::capture_mouse(hwnd);
}

fn on_mouse_move(hwnd: HWND) {
    let drag = with_state(hwnd, |state| state.drag).flatten();
    let Some(drag) = drag else {
        return;
    };

    let Some(cursor) = window::cursor_pos() else {
        return;
    };

    let x = drag.window.left + (cursor.x - drag.cursor.x);
    let y = drag.window.top + (cursor.y - drag.cursor.y);

    window::move_window(hwnd, x, y);
}

fn on_lbutton_up(hwnd: HWND) {
    let _ = with_state(hwnd, |state| {
        state.drag = None;
    });

    window::release_mouse();
}

fn on_tray(hwnd: HWND, lparam: LPARAM) {
    match lparam.0 as u32 {
        WM_RBUTTONUP => {
            // 注意：此处不能持有 &mut AppState。
            if show_menu(hwnd) == MENU_EXIT_ID {
                // 菜单模态循环已经结束，可以安全销毁窗口。
                window::destroy_window(hwnd);
            }
        }

        WM_LBUTTONUP => {
            let _ = with_state(hwnd, |state| state.schedule.flash());

            window::set_alpha(hwnd, 255);
            window::start_timer(hwnd, ANIMATION_TIMER_ID, ANIMATION_INTERVAL_MS);
        }

        _ => {}
    }
}

fn on_command(hwnd: HWND, wparam: WPARAM) {
    let id = wparam.0 & 0xFFFF;
    if id == MENU_EXIT_ID {
        window::destroy_window(hwnd);
    }
}

fn on_destroy(hwnd: HWND) {
    // 窗口句柄此时仍有效，先显式移除托盘图标；Drop 作为兜底。
    let _ = with_state(hwnd, |state| {
        if let Some(tray) = &mut state.tray {
            tray.remove();
        }
    });

    window::stop_timer(hwnd, DATE_TIMER_ID);
    window::stop_timer(hwnd, ANIMATION_TIMER_ID);
    window::post_quit();
}
