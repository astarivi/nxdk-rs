// SPDX-License-Identifier: MIT
#![no_std]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

mod bindings;

pub use bindings::bindings_xboxkrnl as kernel;
pub use bindings::bindings_pbkit as pbkit;
pub use bindings::bindings_SDL as sdl;
pub use bindings::bindings_windows as winapi;

pub mod hal;
pub mod clib;
pub mod lwip;
pub mod nxdk;
pub mod usb;