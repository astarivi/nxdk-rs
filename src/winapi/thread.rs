use nxdk_sys::winapi::GetCurrentThreadId;

pub fn get_current_thread_id() -> u32 {
    unsafe {
        GetCurrentThreadId()
    }
}