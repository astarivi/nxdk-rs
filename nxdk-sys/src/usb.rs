pub mod usbh_lib {
    include!(concat!(env!("OUT_DIR"), "/bindings_usbh_lib.rs"));
}

pub mod usbh_hid {
    include!(concat!(env!("OUT_DIR"), "/bindings_usbh_hid.rs"));
}

pub mod xid_driver {
    include!(concat!(env!("OUT_DIR"), "/bindings_xid_driver.rs"));
}