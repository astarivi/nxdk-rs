// SPDX-License-Identifier: MIT
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

mod bindings;

pub use bindings::bindings_SDL as sdl;
pub use bindings::bindings_lwip as lwip;
pub use bindings::bindings_pbkit as pbkit;
pub use bindings::bindings_windows as winapi;
pub use bindings::bindings_xboxkrnl as kernel;

pub mod clib;
pub mod hal;
pub mod nxdk;
pub mod usb;
