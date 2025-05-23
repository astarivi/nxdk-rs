/* automatically generated by rust-bindgen 0.69.5 */

extern "C" {
    pub fn XReboot();
}
extern "C" {
    #[doc = " Retrieves information persisted by the process that launched the current XBE.\n\n launchDataType will (likely) be one of the LDT_* defines in xboxkrnl.h\n\n Returns non-zero in the case of failure."]
    pub fn XGetLaunchInfo(
        launchDataType: *mut libc::c_ulong,
        launchData: *mut *const libc::c_uchar,
    ) -> libc::c_int;
}
extern "C" {
    #[doc = " Launches an XBE.  Examples of xbePath might be:\n   c:\\\\blah.xbe\n   c:/foo/bar.xbe\n If the XBE is able to be launched, this method will\n not return.  If there is a problem, then it return."]
    pub fn XLaunchXBE(xbePath: *const libc::c_char);
}
extern "C" {
    #[doc = " Launches an XBE and sets the LAUNCH_DATA_PAGE's LaunchData, which is\n retrievable by the newly launched process.\n\n Examples of xbePath might be:\n   c:\\\\blah.xbe\n   c:/foo/bar.xbe\n If the XBE is able to be launched, this method will\n not return.  If there is a problem, then it return."]
    pub fn XLaunchXBEEx(xbePath: *const libc::c_char, launchData: *const libc::c_void);
}
