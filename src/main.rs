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

    // 获取所有有效窗口及其位置
    let mut window_list = windows
        .into_iter()
        .filter(|&hwnd| {
            hwnd != current &&
            IsWindowVisible(hwnd).as_bool() &&
            !IsIconic(hwnd).as_bool()
        })
        .filter_map(|hwnd| {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                Some((hwnd, rect))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // 当前窗口也加入排序列表
    let mut current_rect = RECT::default();
    GetWindowRect(current, &mut current_rect);
    window_list.push((current, current_rect));

    // 排序：按屏幕横向位置（left, top）
    window_list.sort_by_key(|(_, rect)| (rect.left, rect.top));

    // 找到当前窗口的位置
    let idx = window_list.iter().position(|(hwnd, _)| *hwnd == current);
    if let Some(pos) = idx {
        let target_idx = match direction {
            Direction::Left => {
                if pos == 0 {
                    window_list.len() - 1 // 循环到最后
                } else {
                    pos - 1
                }
            }
            Direction::Right => {
                if pos + 1 >= window_list.len() {
                    0 // 循环到最前
                } else {
                    pos + 1
                }
            }
        };
        let target_hwnd = window_list[target_idx].0;
        SetForegroundWindow(target_hwnd);
    }
}


unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    windows.push(hwnd);
    true.into()
}
