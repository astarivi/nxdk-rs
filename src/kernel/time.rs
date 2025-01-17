use core::time::Duration;
use nxdk_sys::kernel::{KeQuerySystemTime, LARGE_INTEGER};

pub const WINDOWS_EPOCH: u64 = 116444736000000000;

/// Represents a system clock based Timer.
pub struct Timer {
    start_time: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer { start_time: query_system_time() }
    }

    /// Returns the time elapsed since this Timer was created.
    pub fn elapsed(&self) -> Duration {
        let current_time = query_system_time();
        let elapsed_ticks = current_time - self.start_time;

        Duration::from_micros(elapsed_ticks / 10)
    }
}

/// Query system time, equivalent to `KeQuerySystemTime`
pub fn query_system_time() -> u64 {
    let mut current_time = LARGE_INTEGER {
        QuadPart: 0
    };

    unsafe {
        KeQuerySystemTime(&mut current_time);
        current_time.QuadPart as u64
    }
}

/// Converts from Windows timestamp to Unix seconds timestamp
pub fn windows_to_unix_timestamp(sys_time: &u64) -> u64 {
    (sys_time - WINDOWS_EPOCH) / 10_000_000
}

/// Gets the unix timestamp from the system clock, in seconds.
pub fn get_unix_timestamp() -> u64 {
    windows_to_unix_timestamp(&query_system_time())
}