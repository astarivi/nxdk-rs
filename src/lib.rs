// SPDX-License-Identifier: MIT
#![no_std]

extern crate alloc;

pub use nxdk_sys as sys;
pub use bitflags;
pub use embedded_io;

pub mod hal;
pub mod nxdk;
pub mod utils;
pub mod xbox_alloc;
pub mod lwip;
pub mod io;
pub mod winapi;
pub mod kernel;