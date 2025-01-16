use nxdk_sys::lwip::*;
use crate::lwip::local_ipv4_to_native;
use crate::lwip::netconn::error::NetconnErr;
use crate::lwip::netconn::NetconnCommon;
use crate::lwip::netconn::tcp::NetconnTcpType;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub enum NetconnUdpType {
    #[default]
    Udp = netconn_type_NETCONN_UDP as isize,
    UdpLite = netconn_type_NETCONN_UDPLITE as isize,
    UdpNoChecksum = netconn_type_NETCONN_UDPNOCHKSUM as isize,
    UdpIpv6 = netconn_type_NETCONN_UDP_IPV6 as isize,
    UdpLiteIpv6 = netconn_type_NETCONN_UDPLITE_IPV6 as isize,
    UdpNoChecksumIpv6 = netconn_type_NETCONN_UDPNOCHKSUM_IPV6 as isize,
}

/// lwip Netconn UDP Rust wrapper.
///
/// This implementation:
///
/// - Isn't thread-safe, but can be sent between threads
/// - Has no IPv6 support (or is untested, at least)
/// - Considers ERR_OK to be a successful call. Any other code is raised as Err
/// - Works with blocking and non-blocking modes (`set_nonblocking(bool)`)
/// - Timeouts aren't implemented; prefer non-blocking calls over timeouts
///
/// Remember to call `nxdk::net::nx_net_init()` before using a Netconn.
/// Failing to do so will result in unexpected behavior.
pub struct NetconnUdp {
    conn: Option<*mut netconn>,
    conn_type: NetconnUdpType,
}

unsafe impl Send for NetconnUdp {}

impl NetconnCommon for NetconnUdp {
    type InnerType = NetconnUdpType;

    fn get_inner(&self) -> Result<*mut netconn, NetconnErr> {
        self.conn.ok_or(NetconnErr::Clsd)
    }

    fn get_type(&self) -> &Self::InnerType {
        &self.conn_type
    }

    fn delete(&mut self) {
        if let Some(conn) = self.conn.take() {
            unsafe {
                netconn_delete(conn);
            }
        };
    }
}

impl NetconnUdp {
    /// Create a new NetconnUdp object of given type.
    ///
    /// Note: Ipv6 support is untested
    pub fn new(netconn_udp_type: NetconnUdpType) -> Result<Self, NetconnErr> {
        let conn = unsafe {
            netconn_new_with_proto_and_callback(
                netconn_udp_type.clone() as i32,
                0,
                None as netconn_callback
            )
        };

        if conn.is_null() {
            return Err(NetconnErr::Mem);
        }

        Ok(Self {
            conn: Some(conn),
            conn_type: netconn_udp_type
        })
    }

    /// Disconnect a netconn from its current peer.
    /// 
    /// API: `UDP`
    pub fn disconnect(&self) -> Result<(), NetconnErr> {
        let err = unsafe {
            netconn_disconnect(
                self.get_inner()?
            )
        };

        if err != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(err));
        }

        Ok(())
    }
}