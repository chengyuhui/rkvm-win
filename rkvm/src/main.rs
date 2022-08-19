use std::{mem::MaybeUninit, ptr, sync::mpsc};

use anyhow::Result;
use once_cell::sync::OnceCell;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            self, CallNextHookEx, DispatchMessageA, GetMessageA, SetCursorPos, SetWindowsHookExW,
            TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MOUSEHOOKSTRUCT,
            WH_KEYBOARD_LL, WH_MOUSE_LL,
        },
    },
};

mod util;

static KEYBOARD_TX: OnceCell<mpsc::SyncSender<KeyboardEvent>> = OnceCell::new();
static MOUSE_TX: OnceCell<mpsc::SyncSender<MouseEvent>> = OnceCell::new();

struct MouseEvent {}

/// Returns whether this event should be eaten.
fn mouse_handler(wparam: usize, x: i32, y: i32) -> bool {
    match wparam as u32 {
        WindowsAndMessaging::WM_MOUSEMOVE => {
            log::info!("WM_MOUSEMOVE({}, {})", x, y);
        }
        WindowsAndMessaging::WM_MOUSEWHEEL => {
            // log::info!("WM_MOUSEWHEEL");
        }
        WindowsAndMessaging::WM_MOUSEHWHEEL => {
            // log::info!("WM_MOUSEHWHEEL");
        }
        WindowsAndMessaging::WM_LBUTTONDOWN => {
            log::info!("WM_LBUTTONDOWN");
        }
        WindowsAndMessaging::WM_LBUTTONUP => {
            log::info!("WM_LBUTTONUP");
        }
        WindowsAndMessaging::WM_RBUTTONDOWN => {
            log::info!("WM_RBUTTONDOWN");
        }
        WindowsAndMessaging::WM_RBUTTONUP => {
            log::info!("WM_RBUTTONUP");
        }
        WindowsAndMessaging::WM_MBUTTONDOWN => {
            log::info!("WM_MBUTTONDOWN");
        }
        WindowsAndMessaging::WM_MBUTTONUP => {
            log::info!("WM_MBUTTONUP");
        }
        _ => {}
    }
    false
}

unsafe extern "system" fn native_mouse_handler(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    }

    let p = if let Some(p) = (lparam.0 as *const MOUSEHOOKSTRUCT).as_ref() {
        p
    } else {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    };

    if mouse_handler(wparam.0, p.pt.x, p.pt.y) {
        LRESULT(1)
    } else {
        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }
}

#[derive(Debug)]
struct KeyboardEvent {
    code: u32,
    time: u32,
}

/// Returns whether this event should be eaten.
fn keyboard_handler(wparam: usize, time: u32, vk_code: u32) -> bool {
    match wparam as u32 {
        WindowsAndMessaging::WM_KEYDOWN => {
            log::info!("WM_KEYDOWN");
            // let _ = tx.send(KeyboardEvent {
            //     code: p.vkCode,
            //     time: p.time,
            // });
        }
        WindowsAndMessaging::WM_KEYUP => {
            log::info!("WM_KEYUP");
            // let _ = tx.send(KeyboardEvent {
            //     code: p.vkCode,
            //     time: p.time,
            // });
        }
        _ => {}
    }
    false
}

unsafe extern "system" fn native_keyboard_handler(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    }

    let p = if let Some(p) = (lparam.0 as *const KBDLLHOOKSTRUCT).as_ref() {
        p
    } else {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    };

    if keyboard_handler(wparam.0, p.time, p.vkCode) {
        LRESULT(1)
    } else {
        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let (kbd_tx, kbd_rx) = mpsc::sync_channel(10);
    KEYBOARD_TX.set(kbd_tx).unwrap();
    std::thread::spawn(move || {
        let mut time: u32 = 0;
        while let Ok(msg) = kbd_rx.recv() {
            if msg.time < time {
                log::info!("Dropping old message");
                continue;
            }
            time = msg.time;
        }
    });

    let (mouse_tx, mouse_rx) = mpsc::sync_channel(10);
    MOUSE_TX.set(mouse_tx).unwrap();
    std::thread::spawn(move || {
        let mut time: u32 = 0;
        while let Ok(msg) = mouse_rx.recv() {}
    });

    let hinstance = unsafe { GetModuleHandleW(PCWSTR::null())? };

    let (mouse_hook_handle, keyboard_hook_handle) = unsafe {
        (
            SetWindowsHookExW(WH_MOUSE_LL, Some(native_mouse_handler), hinstance, 0)?,
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(native_keyboard_handler), hinstance, 0)?,
        )
    };

    loop {
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = GetMessageA(msg.as_mut_ptr(), HWND::default(), 0, 0);
            if bret.0 > 0 {
                TranslateMessage(msg.as_ptr());
                DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }

    unsafe {
        UnhookWindowsHookEx(mouse_hook_handle);
        UnhookWindowsHookEx(keyboard_hook_handle);
    }

    Ok(())
}
