use std::time::Duration;

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Threading::{
            CreateWaitableTimerExW, SetWaitableTimer, WaitForSingleObject,
            INFINITE, TIMER_ALL_ACCESS,
        },
    },
    core::PCWSTR,
};

const CREATE_WAITABLE_TIMER_HIGH_RESOLUTION: u32 = 0x0000_0002;

pub struct WaitableTimer {
    handle: HANDLE,
}

unsafe impl Send for WaitableTimer {}
unsafe impl Sync for WaitableTimer {}

impl WaitableTimer {
    fn try_new() -> Option<Self> {
        let result = unsafe {
            CreateWaitableTimerExW(
                None,
                PCWSTR::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS.0,
            )
        };
        match result {
            Ok(handle) if !handle.is_invalid() => Some(Self { handle }),
            _ => None,
        }
    }

    fn sleep(&self, duration: Duration) {
        if duration.is_zero() {
            return;
        }
        let due_100ns: i64 = -(duration.as_nanos() as i64 / 100).max(1);
        if unsafe { SetWaitableTimer(self.handle, &due_100ns, 0, None, None, false) }.is_err() {
            return;
        }
        unsafe { WaitForSingleObject(self.handle, INFINITE) };
    }
}

impl Drop for WaitableTimer {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

pub enum SleepBackend {
    HighResolution(WaitableTimer),
    Standard,
}

impl SleepBackend {
    pub fn init() -> Self {
        WaitableTimer::try_new().map_or(Self::Standard, Self::HighResolution)
    }

    #[inline]
    pub fn sleep(&self, duration: Duration) {
        match self {
            Self::HighResolution(t) => t.sleep(duration),
            Self::Standard => std::thread::sleep(duration),
        }
    }
}
