// SPDX-License-Identifier: MIT
use nxdk_sys::hal::video::*;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RefreshRate {
    #[default]
    Default = REFRESH_DEFAULT as isize,
    Hz50 = REFRESH_50HZ as isize,
    Hz60 = REFRESH_60HZ as isize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AvPack {
    None = AV_PACK_NONE as isize,
    Standard = AV_PACK_STANDARD as isize,
    Rfu = AV_PACK_RFU as isize,
    Scart = AV_PACK_SCART as isize,
    Hdtv = AV_PACK_HDTV as isize,
    Vga = AV_PACK_VGA as isize,
    SVideo = AV_PACK_SVIDEO as isize,
}

impl AvPack {
    pub fn from_code(code: u32) -> Self {
        match code {
            AV_PACK_STANDARD => AvPack::Standard,
            AV_PACK_RFU => AvPack::Rfu,
            AV_PACK_SCART => AvPack::Scart,
            AV_PACK_HDTV => AvPack::Hdtv,
            AV_PACK_VGA => AvPack::Vga,
            AV_PACK_SVIDEO => AvPack::SVideo,
            _ => AvPack::None,
        }
    }
}

pub fn get_av_pack() -> AvPack {
    let video_adapter = xvideo_get_encoder_settings() & VIDEO_ADAPTER_MASK;

    AvPack::from_code(video_adapter)
}

pub fn xvideo_get_encoder_settings() -> DWORD {
    unsafe { XVideoGetEncoderSettings() }
}

pub fn xvideo_flush_fb() {
    unsafe {
        XVideoFlushFB();
    }
}

pub fn xvideo_set_mode(width: u32, height: u32, bpp: u32, refresh_rate: RefreshRate) -> bool {
    let ret;

    unsafe {
        ret = XVideoSetMode(width as i32, height as i32, bpp as i32, refresh_rate as i32);
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
