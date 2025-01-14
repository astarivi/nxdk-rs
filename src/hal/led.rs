// SPDX-License-Identifier: MIT
use nxdk_sys::hal::led::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum LEDColor {
    #[default]
    Off = _XLEDColor_XLED_OFF as isize,
    Green = _XLEDColor_XLED_GREEN as isize,
    Red = _XLEDColor_XLED_RED as isize,
    Orange = _XLEDColor_XLED_ORANGE as isize,
}

/// Restore system LED control.
pub fn xreset_led() {
    unsafe {
        XResetLED();
    }
}

/// Set the LED to use a four-step loop.
/// For more characteristics, please see https://xboxdevwiki.net/LED.
pub fn xset_custom_led(color1: LEDColor, color2: LEDColor, color3: LEDColor, color4: LEDColor) {
    unsafe {
        XSetCustomLED(color1 as i32, color2 as i32, color3 as i32, color4 as i32);
    }
}