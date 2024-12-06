pub mod config_sector{
    include!(concat!(env!("OUT_DIR"), "/bindings_configsector.rs"));
}

pub mod fatx{
    include!(concat!(env!("OUT_DIR"), "/bindings_fatx.rs"));
}

pub mod format{
    include!(concat!(env!("OUT_DIR"), "/bindings_format.rs"));
}

pub mod mount{
    include!(concat!(env!("OUT_DIR"), "/bindings_mount.rs"));
}

pub mod net{
    include!(concat!(env!("OUT_DIR"), "/bindings_net.rs"));
}

pub mod path{
    include!(concat!(env!("OUT_DIR"), "/bindings_path.rs"));
}

pub mod xbe{
    include!(concat!(env!("OUT_DIR"), "/bindings_xbe.rs"));
}