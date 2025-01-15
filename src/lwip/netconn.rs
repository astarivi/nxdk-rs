use alloc::ffi::CString;
use core::error::Error;
use core::ffi::c_void;
use core::fmt::{Display, Formatter};
use nxdk_sys::lwip::*;
use crate::io::traits::{Read, Write};
use crate::lwip::pbuf::TcpPbuf;
use crate::lwip::{local_ipv4_to_native, native_ipv4_to_local};
use core::fmt;
use core::net::Ipv4Addr;
use core::ptr::null_mut;

/// Error type for Netconn APIs. Names equivalent to enum names. Can be cast to the
/// error number for inter-op.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NetconnErr {
    Ok = err_enum_t_ERR_OK as isize,
    Mem = err_enum_t_ERR_MEM as isize,
    Buf = err_enum_t_ERR_BUF as isize,
    Timeout = err_enum_t_ERR_TIMEOUT as isize,
    Rte = err_enum_t_ERR_RTE as isize,
    InProgress = err_enum_t_ERR_INPROGRESS as isize,
    Val = err_enum_t_ERR_VAL as isize,
    WouldBlock = err_enum_t_ERR_WOULDBLOCK as isize,
    Use = err_enum_t_ERR_USE as isize,
    Already = err_enum_t_ERR_ALREADY as isize,
    IsConn = err_enum_t_ERR_ISCONN as isize,
    Conn = err_enum_t_ERR_CONN as isize,
    If = err_enum_t_ERR_IF as isize,
    Abrt = err_enum_t_ERR_ABRT as isize,
    Rst = err_enum_t_ERR_RST as isize,
    Clsd = err_enum_t_ERR_CLSD as isize,
    Arg = err_enum_t_ERR_ARG as isize
}

impl NetconnErr {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => NetconnErr::Ok,
            -1 => NetconnErr::Mem,
            -2 => NetconnErr::Buf,
            -3 => NetconnErr::Timeout,
            -4 => NetconnErr::Rte,
            -5 => NetconnErr::InProgress,
            -6 => NetconnErr::Val,
            -7 => NetconnErr::WouldBlock,
            -8 => NetconnErr::Use,
            -9 => NetconnErr::Already,
            -10 => NetconnErr::IsConn,
            -11 => NetconnErr::Conn,
            -12 => NetconnErr::If,
            -13 => NetconnErr::Abrt,
            -14 => NetconnErr::Rst,
            -15 => NetconnErr::Clsd,
            -16 => NetconnErr::Arg,
            _ => {
                // Handle any other unexpected values
                panic!("Unknown error code: {}", code);
            }
        }
    }
}

impl From<i32> for NetconnErr {
    fn from(code: i32) -> Self {
        NetconnErr::from_code(code)
    }
}

impl From<i8> for NetconnErr {
    fn from(code: i8) -> Self {
        NetconnErr::from_code(code as i32)
    }
}

impl Error for NetconnErr {}

impl Display for NetconnErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// lwip Netconn type.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub enum NetconnType {
    #[default]
    Tcp = netconn_type_NETCONN_TCP as isize,
    TcpIpv6 = netconn_type_NETCONN_TCP_IPV6 as isize,
    Udp = netconn_type_NETCONN_UDP as isize,
    UdpLite = netconn_type_NETCONN_UDPLITE as isize,
    UdpNoChecksum = netconn_type_NETCONN_UDPNOCHKSUM as isize,
    UdpIpv6 = netconn_type_NETCONN_UDP_IPV6 as isize,
    UdpLiteIpv6 = netconn_type_NETCONN_UDPLITE_IPV6 as isize,
    UdpNoChecksumIpv6 = netconn_type_NETCONN_UDPNOCHKSUM_IPV6 as isize,
    Raw = netconn_type_NETCONN_RAW as isize,
    RawIpv6 = netconn_type_NETCONN_RAW_IPV6 as isize,
    Invalid = netconn_type_NETCONN_INVALID as isize
}

/// lwip Netconn Rust wrapper.
///
/// This implementation:
///
/// - Isn't thread-safe, but can be sent between threads
/// - Has no IPv6 support
/// - Has support for both TCP and UDP
/// - Considers ERR_OK to be a successful call. Any other code is raised as Err
/// - Works with blocking and non-blocking modes (`set_nonblocking(bool)`)
/// - Timeouts aren't implemented; prefer non-blocking over timeout
///
/// Remember to call `nxdk::net::nx_net_init()` before using a Netconn.
/// Failing to do so will result in unexpected behavior.
#[derive(Eq, PartialEq)]
pub struct Netconn {
    conn: Option<*mut netconn>,
    conn_type: NetconnType
}

impl Netconn {
    /// Create new netconn connection
    ///
    /// API: `Common`
    pub fn new(netconn_type: NetconnType) -> Result<Self, NetconnErr> {
        let conn = unsafe {
            netconn_new_with_proto_and_callback(
                netconn_type.clone() as i32,
                0,
                None as netconn_callback
            )
        };

        if conn.is_null() {
            return Err(NetconnErr::Mem);
        }

        Ok(Self {
            conn: Some(conn),
            conn_type: netconn_type
        })
    }

    fn get_inner(&self) -> Result<*mut netconn, NetconnErr> {
        self.conn.ok_or(NetconnErr::Clsd)
    }

    /// Get the local or remote IP address and port of a netconn.
    /// For RAW netconns, this returns the protocol instead of a port!
    ///
    /// Implementation note: This method returns a raw ip_addr_t, intended
    /// for inter-op with other lwip functions. Prefer `peer()` or `addr()` over this
    /// for other use cases.
    /// 
    /// API: `Common`
    pub fn getaddr_native(&self, local: bool) -> Result<(ip_addr_t, u16), NetconnErr> {
        let mut ipaddr: ip_addr_t = unsafe { core::mem::zeroed() };
        let mut port: u16 = 0;

        let result = unsafe {
            netconn_getaddr(
                self.get_inner()?,
                &mut ipaddr,
                &mut port,
                if local {1} else {0}
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok((ipaddr, port))
    }

    /// Get the remote peer IP address and port of a netconn.
    /// For RAW netconns, this returns the protocol instead of a port!
    ///
    /// API: `Common`
    pub fn peer(&self) -> Result<(Ipv4Addr, u16), NetconnErr> {
        let ipaddr = self.getaddr_native(false)?;
        Ok((native_ipv4_to_local(&ipaddr.0), ipaddr.1))
    }

    /// Get the local IP address and port of a netconn.
    /// For RAW netconns, this returns the protocol instead of a port!
    ///
    /// API: `Common`
    pub fn addr(&self) -> Result<(Ipv4Addr, u16), NetconnErr> {
        let ipaddr = self.getaddr_native(true)?;
        Ok((native_ipv4_to_local(&ipaddr.0), ipaddr.1))
    }

    /// Close a netconn 'connection' and free all its resources but
    /// not the netconn itself. UDP and RAW connection are completely
    /// closed, TCP pcbs might still be in a waitstate after this returns.
    ///
    /// API: `Common`
    pub fn prepare_delete(&self) -> Result<(), NetconnErr> {
        let result = unsafe {
            netconn_prepare_delete(
                self.get_inner()?,
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(())
    }

    /// Close a netconn 'connection' and free its resources.
    /// UDP and RAW connection are completely closed,
    /// TCP pcbs might still be in a waitstate after this returns.
    ///
    /// Implementation note: Further calls to a deleted Netconn will
    /// always yield error `Clsd`
    /// 
    /// API: `Common`
    pub fn delete(&mut self) {
        if let Some(conn) = self.conn.take() {
            unsafe {
                netconn_delete(conn);
            }
        };
    }

    /// Bind a netconn to a specific local IP address and port.
    /// Binding one netconn twice might not always be checked correctly!
    ///
    /// Implementation note: This binds to all addresses.
    /// API: `Common`
    pub fn bind(&mut self, port: u16) -> Result<(), NetconnErr> {
        let result = unsafe {
            netconn_bind(
                self.get_inner()?,
                null_mut(),
                port
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(())
    }

    /// Bind a netconn to a specific local IP address and port.
    /// Binding one netconn twice might not always be checked correctly!
    ///
    /// Implementation note: This binds only to the given address.
    /// API: `Common`
    pub fn bind_to(&mut self, ip: &Ipv4Addr, port: u16) -> Result<(), NetconnErr> {
        let result = unsafe {
            netconn_bind(
                self.get_inner()?,
                &mut local_ipv4_to_native(&ip),
                port
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(())
    }

    /// Connect a netconn to a specific remote IP address and port.
    ///
    /// Implementation note: This binds only to the given address.
    /// API: `Common`
    pub fn connect(&mut self, addr: &Ipv4Addr, port: u16) -> Result<(), NetconnErr> {
        let err = unsafe {
            netconn_connect(
                self.get_inner()?,
                &local_ipv4_to_native(addr) as *const ip_addr,
                port
            )
        };

        if err != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(err));
        }

        Ok(())
    }

    /// Get and reset pending error on a netconn
    /// 
    /// API: `Common`
    pub fn err(&self) -> NetconnErr {
        let conn: *mut netconn = match self.get_inner() {
            Ok(x) => {x}
            Err(e) => {
                return e;
            }
        };

        NetconnErr::from(unsafe {
            netconn_err(
                conn
            )
        })
    }

    /// Set the non-blocking flag for this netconn to the given value. Default is false.
    ///
    /// API: `Common`
    pub fn set_nonblocking(&self, value: bool) -> Result<(), NetconnErr> {
        if value {
            unsafe {
                let inner_con = self.get_inner()?;
                (*inner_con).flags |= NETCONN_FLAG_NON_BLOCKING as u8;
            }
        } else {
            unsafe {
                let inner_con = self.get_inner()?;
                (*inner_con).flags &= !(NETCONN_FLAG_NON_BLOCKING as u8);
            }
        }

        Ok(())
    }

    /// Set a TCP netconn into listen mode
    /// 
    /// API: `TCP`
    pub fn listen(&mut self) -> Result<(), NetconnErr> {
        let _new_pcb = unsafe {
            netconn_listen_with_backlog(
                self.get_inner()?,
                TCP_DEFAULT_LISTEN_BACKLOG as u8
            )
        };

        Ok(())
    }

    /// Accept a new connection on a TCP listening netconn
    ///
    /// API: `TCP`
    pub fn accept(&mut self) -> Result<Option<Self>, NetconnErr> {
        let mut new_conn: *mut netconn = null_mut();

        let result = unsafe {
            netconn_accept(
                self.get_inner()?,
                &mut new_conn
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        if new_conn.is_null() {
            return Ok(None)
        }

        Ok(Some(Self {
            conn: Some(new_conn),
            conn_type: self.conn_type.clone()
        }))
    }

    pub fn read_no_copy(&mut self) -> Result<TcpPbuf, NetconnErr>{
        let mut pbuf_ptr: *mut pbuf = null_mut();

        let err = unsafe {
            netconn_recv_tcp_pbuf(
                self.get_inner()?,
                &mut pbuf_ptr
            )
        };

        if err != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(err));
        }

        if pbuf_ptr.is_null() {
            return Err(NetconnErr::Mem);
        }

        Ok(TcpPbuf::new(pbuf_ptr))
    }
    
    pub fn close_and_delete(&mut self) {
        if let Some(conn) = self.conn.take() {
            unsafe {
                netconn_close(conn);
                netconn_delete(conn);
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(conn) = self.conn.take() {
            unsafe {
                netconn_close(conn);
            }
        }
    }
}

impl Write for Netconn {
    type WriteError = NetconnErr;

    // FIXME: Box the buffer for calls that aren't blocking.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::WriteError> {
        let mut bytes_written: usize = 0;

        let result = unsafe {
            netconn_write_partly(
                self.get_inner()?,
                buf.as_ptr() as *const c_void,
                buf.len(),
                NETCONN_COPY as u8,
                &mut bytes_written,
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        unimplemented!();
    }
}

impl Read for Netconn {
    type ReadError = NetconnErr;

    /// This is an unoptimized way of reading that copies data to the given buffer.
    /// If there's more data available than it fits in given buf, it's discarded.
    /// 
    /// ## Use read_no_copy() instead of this method for large transfers
    /// 
    /// Implementation note: This is equivalent to `netbuff_recv`
    /// 
    /// API: `Common`
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::ReadError> {
        let mut pbuf = self.read_no_copy()?;
        let mut total_written: usize = 0;

        while let Some(chunk) = pbuf.next() {
            let remaining_space = buf.len().saturating_sub(total_written);

            if remaining_space == 0 {
                pbuf.close();
                return Ok(total_written);
            }

            let copy_len = core::cmp::min(chunk.len(), remaining_space);
            buf[total_written..total_written + copy_len].copy_from_slice(&chunk[..copy_len]);
            total_written += copy_len;
        };

        Ok(total_written)
    }
}

unsafe impl Send for Netconn {}

pub fn get_host_by_name_native(hostname: &str) -> Result<ip_addr_t, NetconnErr>{
    let mut ipaddr: ip_addr_t = unsafe { core::mem::zeroed() };
    let hostname_c = CString::new(hostname).map_err(|e| NetconnErr::Mem)?;

    let err = unsafe {
        netconn_gethostbyname_addrtype(
            hostname_c.as_ptr(),
            &mut ipaddr,
            NETCONN_DNS_IPV4 as u8
        )
    };

    if err != err_enum_t_ERR_OK as i8 {
        return Err(NetconnErr::from(err));
    }

    Ok(ipaddr)
}

pub fn get_host_by_name(hostname: &str) -> Result<Ipv4Addr, NetconnErr>{
    Ok(native_ipv4_to_local(&get_host_by_name_native(hostname)?))
}