use std::collections::HashMap;
use std::ptr::null_mut;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::*,
        Graphics::Gdi::{MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO},
        System::Threading::GetCurrentThreadId,
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::*,
        },
    },
};

const HOTKEY_LEFT: i32 = 1;
const HOTKEY_RIGHT: i32 = 2;

fn main() {
    unsafe {
        // 注册热键
        RegisterHotKey(None, HOTKEY_LEFT, MOD_ALT | MOD_SHIFT, 'H' as u32);
        RegisterHotKey(None, HOTKEY_RIGHT, MOD_ALT | MOD_SHIFT, 'L' as u32);

        println!("Running... Press ALT+SHIFT+H/L to switch window.");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                match msg.wParam.0 as i32 {
                    HOTKEY_LEFT => switch_window(Direction::Left),
                    HOTKEY_RIGHT => switch_window(Direction::Right),
                    _ => {}
                }
            }
        }

        // 程序退出时注销热键
        UnregisterHotKey(None, HOTKEY_LEFT);
        UnregisterHotKey(None, HOTKEY_RIGHT);
    }
}

#[derive(PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

unsafe fn switch_window(direction: Direction) {
    let current = GetForegroundWindow();
    if current.0 == 0 || IsIconic(current).as_bool() {
        return;
    }

    let mut windows = Vec::new();
    EnumWindows(Some(enum_window_proc), LPARAM(&mut windows as *mut _ as isize));

    let current_rect = {
        let mut rect = RECT::default();
        GetWindowRect(current, &mut rect);
        rect
    };
    let current_monitor = MonitorFromWindow(current, MONITOR_DEFAULTTONEAREST);

    // 找到在当前显示器上的、非最小化、可见窗口
    let mut candidates = windows
        .into_iter()
        .filter(|&hwnd| {
            hwnd != current &&
            IsWindowVisible(hwnd).as_bool() &&
            !IsIconic(hwnd).as_bool() &&
            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) == current_monitor
        })
        .collect::<Vec<_>>();

    // 获取候选窗口的坐标
    let mut target: Option<(HWND, i32)> = None;
    for hwnd in candidates {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).as_bool() {
            let dx = rect.left - current_rect.left;
            match direction {
                Direction::Left if dx < 0 => {
                    let dist = dx.abs();
                    if target.is_none() || dist < target.unwrap().1 {
                        target = Some((hwnd, dist));
                    }
                }
                Direction::Right if dx > 0 => {
                    let dist = dx;
                    if target.is_none() || dist < target.unwrap().1 {
                        target = Some((hwnd, dist));
                    }
                }
                _ => {}
            }
        }
    }

    if let Some((target_hwnd, _)) = target {
        SetForegroundWindow(target_hwnd);
    }
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    windows.push(hwnd);
    true.into()
}
