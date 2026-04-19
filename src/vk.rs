use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY,
    VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8,
    VK_F9, VK_F10, VK_F11, VK_F12,
    VK_CAPITAL, VK_TAB, VK_INSERT, VK_DELETE,
    VK_HOME, VK_END, VK_PRIOR, VK_NEXT,
    VK_NUMLOCK, VK_SCROLL,
    VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU,
};

use crate::error::ConfigError;

pub fn from_str(name: &str) -> Result<VIRTUAL_KEY, ConfigError> {
    let upper = name.trim().to_uppercase();
    let vk = match upper.as_str() {
        "F1" => VK_F1, "F2" => VK_F2, "F3" => VK_F3, "F4" => VK_F4,
        "F5" => VK_F5, "F6" => VK_F6, "F7" => VK_F7, "F8" => VK_F8,
        "F9" => VK_F9, "F10" => VK_F10, "F11" => VK_F11, "F12" => VK_F12,
        "CAPS" | "CAPSLOCK" => VK_CAPITAL,
        "TAB" => VK_TAB,
        "INSERT" => VK_INSERT,
        "DELETE" | "DEL" => VK_DELETE,
        "HOME" => VK_HOME,
        "END" => VK_END,
        "PAGEUP" | "PGUP" => VK_PRIOR,
        "PAGEDOWN" | "PGDN" => VK_NEXT,
        "NUMLOCK" => VK_NUMLOCK,
        "SCROLLLOCK" => VK_SCROLL,
        "SHIFT" | "LSHIFT" => VK_LSHIFT,
        "RSHIFT" => VK_RSHIFT,
        "CTRL" | "LCTRL" => VK_LCONTROL,
        "RCTRL" => VK_RCONTROL,
        "ALT" | "LALT" => VK_LMENU,
        "RALT" => VK_RMENU,
        s if s.len() == 1 && s.as_bytes()[0].is_ascii_alphanumeric() => {
            VIRTUAL_KEY(s.as_bytes()[0] as u16)
        }
        _ => return Err(ConfigError::UnknownKey(name.to_owned())),
    };
    Ok(vk)
}

#[inline]
pub fn is_held(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk.0 as i32) } as u16) & 0x8000 != 0
}
