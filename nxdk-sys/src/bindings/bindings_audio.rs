/* automatically generated by rust-bindgen 0.64.0 */

pub type XAudioCallback = ::core::option::Option<
    unsafe extern "C" fn(pac97Device: *mut libc::c_void, data: *mut libc::c_void),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AC97_DESCRIPTOR {
    pub bufferStartAddress: libc::c_uint,
    pub bufferLengthInSamples: libc::c_ushort,
    pub bufferControl: libc::c_ushort,
}
#[test]
fn bindgen_test_layout_AC97_DESCRIPTOR() {
    const UNINIT: ::core::mem::MaybeUninit<AC97_DESCRIPTOR> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<AC97_DESCRIPTOR>(),
        8usize,
        concat!("Size of: ", stringify!(AC97_DESCRIPTOR))
    );
    assert_eq!(
        ::core::mem::align_of::<AC97_DESCRIPTOR>(),
        4usize,
        concat!("Alignment of ", stringify!(AC97_DESCRIPTOR))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).bufferStartAddress) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DESCRIPTOR),
            "::",
            stringify!(bufferStartAddress)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).bufferLengthInSamples) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DESCRIPTOR),
            "::",
            stringify!(bufferLengthInSamples)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).bufferControl) as usize - ptr as usize },
        6usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DESCRIPTOR),
            "::",
            stringify!(bufferControl)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AC97_DEVICE {
    pub pcmSpdifDescriptor: [AC97_DESCRIPTOR; 32usize],
    pub pcmOutDescriptor: [AC97_DESCRIPTOR; 32usize],
    pub mmio: *mut libc::c_uint,
    pub nextDescriptor: libc::c_uchar,
    pub callback: XAudioCallback,
    pub callbackData: *mut libc::c_void,
    pub sampleSizeInBits: libc::c_int,
    pub numChannels: libc::c_int,
}
#[test]
fn bindgen_test_layout_AC97_DEVICE() {
    const UNINIT: ::core::mem::MaybeUninit<AC97_DEVICE> = ::core::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::core::mem::size_of::<AC97_DEVICE>(),
        536usize,
        concat!("Size of: ", stringify!(AC97_DEVICE))
    );
    assert_eq!(
        ::core::mem::align_of::<AC97_DEVICE>(),
        8usize,
        concat!("Alignment of ", stringify!(AC97_DEVICE))
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pcmSpdifDescriptor) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(pcmSpdifDescriptor)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).pcmOutDescriptor) as usize - ptr as usize },
        256usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(pcmOutDescriptor)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).mmio) as usize - ptr as usize },
        512usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(mmio)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).nextDescriptor) as usize - ptr as usize },
        516usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(nextDescriptor)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).callback) as usize - ptr as usize },
        520usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(callback)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).callbackData) as usize - ptr as usize },
        524usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(callbackData)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).sampleSizeInBits) as usize - ptr as usize },
        528usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(sampleSizeInBits)
        )
    );
    assert_eq!(
        unsafe { ::core::ptr::addr_of!((*ptr).numChannels) as usize - ptr as usize },
        532usize,
        concat!(
            "Offset of field: ",
            stringify!(AC97_DEVICE),
            "::",
            stringify!(numChannels)
        )
    );
}
extern "C" {
    pub fn XAudioInit(
        sampleSizeInBits: libc::c_int,
        numChannels: libc::c_int,
        callback: XAudioCallback,
        data: *mut libc::c_void,
    );
}
extern "C" {
    pub fn XAudioPlay();
}
extern "C" {
    pub fn XAudioPause();
}
extern "C" {
    pub fn XAudioProvideSamples(
        buffer: *mut libc::c_uchar,
        bufferLength: libc::c_ushort,
        isFinal: libc::c_int,
    );
}