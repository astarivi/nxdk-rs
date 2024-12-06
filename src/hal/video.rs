// SPDX-License-Identifier: MIT

use nxdk_sys::hal::video::*;

#[derive(Debug, Default)]
pub enum RefreshRate {
    #[default]
    Default = REFRESH_DEFAULT as isize,
    Hz50 = REFRESH_50HZ as isize,
    Hz60 = REFRESH_60HZ as isize,
}

pub fn xvideo_flush_fb() {
    unsafe {
        XVideoFlushFB();
    }
}

// TODO: Change signed values to unsigned.
pub fn xvideo_set_mode(width: i32, height: i32, bpp: i32, refresh_rate: RefreshRate) -> bool {
    let ret;

    unsafe {
        ret = XVideoSetMode(width, height, bpp, refresh_rate as i32);
    }

    ret != 0
}

pub fn xvideo_set_soften_filter(enable: bool) {
    unsafe {
        XVideoSetSoftenFilter(enable as i32);
    }
}

pub fn xvideo_set_video_enable(enable: bool) {
    unsafe {
        XVideoSetVideoEnable(enable as i32);
    }
}

pub fn xvideo_wait_for_vblank() {
    unsafe {
        XVideoWaitForVBlank();
    }
}