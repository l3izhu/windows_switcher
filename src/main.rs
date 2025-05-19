use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::{
            Diagnostics::ToolHelp::*,
            LibraryLoader::*,
            Threading::{GetCurrentThreadId, GetWindowThreadProcessId},
        },
        UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, MOD_SHIFT, VK_H, VK_L, keybd_event, VK_MENU, KEYEVENTF_KEYUP},
            WindowsAndMessaging::*,
        },
    },
};

#[derive(PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

fn main() -> Result<()> {
    unsafe {
        RegisterHotKey(None, 1, MOD_ALT | MOD_SHIFT, VK_H.0 as u32);
        RegisterHotKey(None, 2, MOD_ALT | MOD_SHIFT, VK_L.0 as u32);

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                match msg.wParam.0 {
                    1 => switch_window(Direction::Left),
                    2 => switch_window(Direction::Right),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    
    // 排除非顶层窗口
    if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() || GetParent(hwnd).0 != 0 {
        return TRUE;
    }

    let exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    if exstyle & WS_EX_TOOLWINDOW.0 as isize != 0 {
        return TRUE; // 排除工具窗口
    }

    windows.push(hwnd);
    TRUE
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
        .filter(|&hwnd| hwnd != current)
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
    if GetWindowRect(current, &mut current_rect).is_ok() {
        window_list.push((current, current_rect));
    }

    // 排序：按屏幕位置 (left, top)
    window_list.sort_by_key(|(_, rect)| (rect.left, rect.top));

    // 找到当前窗口位置
    let idx = window_list.iter().position(|(hwnd, _)| *hwnd == current);
    if let Some(pos) = idx {
        let target_idx = match direction {
            Direction::Left => if pos == 0 { window_list.len() - 1 } else { pos - 1 },
            Direction::Right => if pos + 1 >= window_list.len() { 0 } else { pos + 1 },
        };
        let target_hwnd = window_list[target_idx].0;
        force_activate_window(target_hwnd);
    }
}

unsafe fn force_activate_window(hwnd: HWND) {
    // 模拟 ALT 键，绕过前台窗口限制
    keybd_event(VK_MENU.0 as u8, 0, 0, 0);
    SetForegroundWindow(hwnd);
    keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
}
