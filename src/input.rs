use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};

#[derive(Clone, Copy)]
pub enum Button {
    Left,
    Right,
}

impl Button {
    #[inline]
    fn flags(self) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
        match self {
            Button::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            Button::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        }
    }
}

pub fn send_click(button: Button) {
    let (down, up) = button.flags();
    send_inputs(&[make_mouse_input(down), make_mouse_input(up)]);
}

pub fn release_all_buttons() {
    send_inputs(&[make_mouse_input(MOUSEEVENTF_LEFTUP), make_mouse_input(MOUSEEVENTF_RIGHTUP)]);
}

fn make_mouse_input(flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT { dwFlags: flags, ..Default::default() },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
}
