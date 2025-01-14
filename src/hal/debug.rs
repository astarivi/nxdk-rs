// SPDX-License-Identifier: MIT
extern crate alloc;

use alloc::ffi::CString;
use alloc::format;
use core::ffi::CStr;
use nxdk_sys::hal::debug::*;

pub fn debug_print_cstr(msg: &CStr) {
    unsafe {
        debugPrint(msg.as_ptr() as *const libc::c_char);
    }
}

/// Prints a message to whatever debug facilities might be available, this is usually the screen
/// and terminal. Fails silently if the strings fails to encode.
pub fn debug_print_str(msg: &str) {
    if let Ok(cstr) = CString::new(msg) {
        debug_print_cstr(&cstr);
    }
}

// Implementations from the-pink-hacker
pub fn debug_print_str_ln(msg: &str) {
    debug_print_str(&format!("{}\n", msg));
}

pub fn debug_print_number(number: impl Into<i32>) {
    let into = number.into();

    unsafe {
        debugPrintNum(into);
    }
}

pub fn debug_print_binary(number: impl Into<i32>) {
    let into = number.into();

    unsafe {
        debugPrintBinary(into);
    }
}

pub fn debug_print_hex_cstr(msg: &CStr, length: u32) {
    unsafe {
        debugPrintHex(msg.as_ptr() as *const libc::c_char, length as i32);
    }
}

pub fn debug_print_hex_str(msg: &str, length: u32) {
    if let Ok(cstr) = CString::new(msg) {
        debug_print_hex_cstr(&cstr, length);
    }
}

pub fn debug_clear_screen() {
    unsafe {
        debugClearScreen();
    }
}

pub fn debug_advance_screen() {
    unsafe {
        debugAdvanceScreen();
    }
}

pub fn debug_move_cursor(x: u32, y: u32) {
    // TODO: add bounds checking
    unsafe {
        debugMoveCursor(x as i32, y as i32);
    }
}

pub fn debug_reset_cursor() {
    unsafe {
        debugResetCursor();
    }
}
