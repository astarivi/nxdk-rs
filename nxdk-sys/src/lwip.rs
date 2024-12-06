pub mod api {
    include!(concat!(env!("OUT_DIR"), "/bindings_api.rs"));
}

pub mod opt {
    include!(concat!(env!("OUT_DIR"), "/bindings_opt.rs"));
}

pub mod netif {
    include!(concat!(env!("OUT_DIR"), "/bindings_netif.rs"));
}

pub mod tcpip {
    include!(concat!(env!("OUT_DIR"), "/bindings_tcpip.rs"));
}