use alloc::ffi::CString;
use core::net::Ipv4Addr;
use core::ptr::null_mut;
use nxdk_sys::lwip::*;
use crate::lwip::{local_ipv4_to_native, native_ipv4_to_local};
use crate::lwip::netconn::error::NetconnErr;

pub mod tcp;
pub mod udp;
pub mod error;

pub trait NetconnCommon {
    type InnerType;

    fn get_inner(&self) -> Result<*mut netconn, NetconnErr>;
    
    fn get_type(&self) -> &Self::InnerType;

    /// Get the local or remote IP address and port of a netconn.
    /// For RAW netconns, this returns the protocol instead of a port!
    ///
    /// Implementation note: This method returns a raw ip_addr_t, intended
    /// for inter-op with other lwip functions. Prefer `peer()` or `addr()` over this
    /// for other use cases.
    ///
    /// API: `Common`
    fn getaddr_native(&self, local: bool) -> Result<(ip_addr_t, u16), NetconnErr> {
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
    fn peer(&self) -> Result<(Ipv4Addr, u16), NetconnErr> {
        let ipaddr = self.getaddr_native(false)?;
        Ok((native_ipv4_to_local(&ipaddr.0), ipaddr.1))
    }

    /// Get the local IP address and port of a netconn.
    /// For RAW netconns, this returns the protocol instead of a port!
    ///
    /// API: `Common`
    fn addr(&self) -> Result<(Ipv4Addr, u16), NetconnErr> {
        let ipaddr = self.getaddr_native(true)?;
        Ok((native_ipv4_to_local(&ipaddr.0), ipaddr.1))
    }

    /// Close a netconn 'connection' and free all its resources but
    /// not the netconn itself. UDP and RAW connection are completely
    /// closed, TCP pcbs might still be in a waitstate after this returns.
    ///
    /// API: `Common`
    fn prepare_delete(&self) -> Result<(), NetconnErr> {
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
    fn delete(&mut self);

    /// Bind a netconn to a specific local IP address and port.
    /// Binding one netconn twice might not always be checked correctly!
    ///
    /// Implementation note: This binds to all addresses.
    /// API: `Common`
    fn bind(&mut self, port: u16) -> Result<(), NetconnErr> {
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
    fn bind_to(&mut self, ip: &Ipv4Addr, port: u16) -> Result<(), NetconnErr> {
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
    fn connect(&mut self, addr: &Ipv4Addr, port: u16) -> Result<(), NetconnErr> {
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
    fn err(&self) -> NetconnErr {
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
    fn set_nonblocking(&self, value: bool) -> Result<(), NetconnErr> {
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
}


/// Execute a DNS query, only one IP address is returned
/// 
/// Native version of `get_host_by_name`, returns a `ip_addr_t` for
/// inter-op use cases.
///
/// Note: This method doesn't require a Netconn object, but
/// `nxdk::net::nx_net_init()` must have been called before
/// this fn. Failure to do so will result in deadlocks.
///
/// API: `Common`
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

/// Execute a DNS query, only one IP address is returned
/// 
/// Note: This method doesn't require a Netconn object, but
/// `nxdk::net::nx_net_init()` must have been called before
/// this fn. Failure to do so will result in deadlocks.
/// 
/// API: `Common`
pub fn get_host_by_name(hostname: &str) -> Result<Ipv4Addr, NetconnErr>{
    Ok(native_ipv4_to_local(&get_host_by_name_native(hostname)?))
}