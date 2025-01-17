// SPDX-License-Identifier: MIT
#![no_std]

extern crate alloc;

pub use nxdk_sys as sys;
pub mod hal;
pub mod nxdk;
pub mod utils;
pub mod xbox_alloc;
pub mod lwip;
pub mod io;
pub mod winapi;