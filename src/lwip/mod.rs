use core::net::Ipv4Addr;
use nxdk_sys::lwip::*;

pub mod netconn;
pub mod pbuf;

pub fn native_ipv4_to_local(ipaddr: &ip_addr_t) -> Ipv4Addr {
    unsafe {
        Ipv4Addr::new(
            (ipaddr.u_addr.ip4.addr & 0xFF) as u8,
            ((ipaddr.u_addr.ip4.addr >> 8) & 0xFF) as u8,
            ((ipaddr.u_addr.ip4.addr >> 16) & 0xFF) as u8,
            ((ipaddr.u_addr.ip4.addr >> 24) & 0xFF) as u8
        )
    }
}

pub fn local_ipv4_to_native(ipv4: &Ipv4Addr) -> ip_addr_t{
    ip_addr {
        u_addr: ip_addr__bindgen_ty_1 {
            ip4: ip4_addr {
                addr: u32::from_le_bytes(ipv4.octets()),
            }
        },
        type_: lwip_ip_addr_type_IPADDR_TYPE_V4 as u8,
    }
}