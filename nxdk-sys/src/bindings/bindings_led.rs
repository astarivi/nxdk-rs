/* automatically generated by rust-bindgen 0.64.0 */

pub const _XLEDColor_XLED_OFF: _XLEDColor = 0;
pub const _XLEDColor_XLED_GREEN: _XLEDColor = 1;
pub const _XLEDColor_XLED_RED: _XLEDColor = 16;
pub const _XLEDColor_XLED_ORANGE: _XLEDColor = 17;
pub type _XLEDColor = libc::c_int;
pub use self::_XLEDColor as XLEDColor;
extern "C" {
    pub fn XResetLED();
}
extern "C" {
    pub fn XSetCustomLED(t1: XLEDColor, t2: XLEDColor, t3: XLEDColor, t4: XLEDColor);
}