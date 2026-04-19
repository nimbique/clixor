use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
            HHOOK, LLMHF_INJECTED, MSLLHOOKSTRUCT, MSG, WH_MOUSE_LL,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
        },
    },
    core::PCWSTR,
};

use crate::error::ClixorError;

static LMB_HELD: AtomicBool = AtomicBool::new(false);
static RMB_HELD: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
        if info.flags & LLMHF_INJECTED == 0 {
            match wparam.0 as u32 {
                WM_LBUTTONDOWN => LMB_HELD.store(true, Ordering::Relaxed),
                WM_LBUTTONUP => LMB_HELD.store(false, Ordering::Relaxed),
                WM_RBUTTONDOWN => RMB_HELD.store(true, Ordering::Relaxed),
                WM_RBUTTONUP => RMB_HELD.store(false, Ordering::Relaxed),
                _ => {}
            }
        }
    }
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

pub struct MouseHook;

impl MouseHook {
    pub fn install() -> Result<Self, ClixorError> {
        let (tx, rx) = mpsc::channel::<Result<(), String>>();
        thread::Builder::new()
            .name("mouse_hook_msg_loop".into())
            .spawn(move || Self::message_loop(tx))
            .map_err(|e| ClixorError::HookInstall(e.to_string()))?;
        rx.recv()
            .map_err(|_| ClixorError::HookInstall("hook thread died before signaling".into()))?
            .map_err(ClixorError::HookInstall)?;
        Ok(Self)
    }

    fn message_loop(ready: mpsc::Sender<Result<(), String>>) {
        unsafe {
            let hmod = match GetModuleHandleW(PCWSTR::null()) {
                Ok(h) => h,
                Err(e) => {
                    let _ = ready.send(Err(e.to_string()));
                    return;
                }
            };
            let hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hmod, 0) {
                Ok(h) => h,
                Err(e) => {
                    let _ = ready.send(Err(e.to_string()));
                    return;
                }
            };
            let _ = ready.send(Ok(()));
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).0 > 0 {}
            let _ = UnhookWindowsHookEx(hook);
        }
    }

    #[inline]
    pub fn lmb_held(&self) -> bool {
        LMB_HELD.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn rmb_held(&self) -> bool {
        RMB_HELD.load(Ordering::Relaxed)
    }
}
