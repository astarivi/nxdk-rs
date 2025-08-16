use crate::lwip::netconn::error::NetconnErr;
use crate::lwip::netconn::NetconnCommon;
use crate::lwip::pbuf::TcpPbuf;
use core::ffi::c_void;
use core::ptr::null_mut;
use nxdk_sys::lwip::*;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub enum NetconnTcpType {
    #[default]
    Tcp = netconn_type_NETCONN_TCP as isize,
    TcpIpv6 = netconn_type_NETCONN_TCP_IPV6 as isize
}

/// lwip Netconn TCP Rust wrapper.
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
#[derive(Debug)]
pub struct NetconnTcp {
    conn: Option<*mut netconn>,
    conn_type: NetconnTcpType,
}

unsafe impl Send for NetconnTcp {}

impl NetconnCommon for NetconnTcp {
    type InnerType = NetconnTcpType;

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

impl NetconnTcp {
    /// Create a new NetconnTcp socket of given type.
    ///
    /// Note: Ipv6 support is untested
    pub fn new(netconn_tcp_type: NetconnTcpType) -> Result<Self, NetconnErr> {
        let conn = unsafe {
            netconn_new_with_proto_and_callback(
                netconn_tcp_type.clone() as i32,
                0,
                None as netconn_callback
            )
        };

        if conn.is_null() {
            return Err(NetconnErr::Mem);
        }

        Ok(Self {
            conn: Some(conn),
            conn_type: netconn_tcp_type
        })
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

    /// Equivalent to calling: `netconn_recv_tcp_pbuf`
    ///
    /// Keep in mind that if an error is returned, it may not always
    /// mean a network error; no data may have been received since
    /// the last call.
    ///
    /// Receive data (in the form of a pbuf) from a TCP netconn
    ///
    /// API: `TCP`
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

    pub async fn read_no_copy_async(&mut self) -> Result<TcpPbuf, NetconnErr> {
        self.set_nonblocking(true)?;

        loop {
            return match self.read_no_copy() {
                Ok(x) => {
                    Ok(x)
                }
                Err(e) => {
                    if e == NetconnErr::WouldBlock {
                        futures_lite::future::yield_now().await;
                        continue;
                    }

                    Err(e)
                }
            }
        }

    }

    /// Shut down one or both sides of a TCP netconn (doesn't delete it).
    ///
    /// # Arguments
    ///
    /// - `rx_side` shut down the RX side (no more read possible after this)
    /// - `tx_side` shut down the TX side (no more write possible after this)
    ///
    /// API: `TCP`
    pub fn shutdown(&self, rx_side: bool, tx_side: bool) -> Result<(), NetconnErr> {
        let result = unsafe {
            netconn_shutdown(
                self.get_inner()?,
                rx_side as u8,
                tx_side as u8
            )
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(())
    }

    /// Close a TCP netconn (doesn't delete it).
    ///
    /// API: `TCP`
    pub fn close(&mut self) -> Result<(), NetconnErr> {
        let result = unsafe {
            netconn_close(self.get_inner()?)
        };

        if result != err_enum_t_ERR_OK as i8 {
            return Err(NetconnErr::from(result));
        }

        Ok(())
    }

    /// Close and delete a TCP connection.
    ///
    /// API: `Util`
    pub fn close_and_delete(&mut self) {
        let _ = self.close();
        self.delete();
    }
}

impl embedded_io::ErrorType for NetconnTcp {
    type Error = NetconnErr;
}

impl embedded_io::Write for NetconnTcp {
    // FIXME: Box the buffer for calls that aren't blocking.
    /// Send data over a TCP netconn.
    ///
    /// This implementation uses the NETCONN_COPY flag, keep that in mind
    /// when choosing a buffer size.
    /// 
    /// API: `TCP`
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
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

    /// Flushing is not supported. Calling this will result in a panic.
    /// 
    /// API: `Unimplemented`
    fn flush(&mut self) -> Result<(), Self::Error> {
        unimplemented!();
    }
}

impl embedded_io::Read for NetconnTcp {
    /// This is an unoptimized way of reading that copies data to the given buffer.
    /// If there's more data available than it fits in a given buf, it's discarded.
    ///
    /// ## Use read_no_copy() instead of this method for large transfers
    ///
    /// API: `TCP`
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
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

impl embedded_io_async::Write for NetconnTcp {

    // FIXME: Box the buffer for calls that aren't blocking.
    /// Send data over a TCP netconn.
    ///
    /// This implementation uses the NETCONN_COPY flag, keep that in mind
    /// when choosing a buffer size.
    ///
    /// API: `TCP`
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.set_nonblocking(true)?;
        let mut bytes_written: usize = 0;

        loop {
            let result = unsafe {
                netconn_write_partly(
                    self.get_inner()?,
                    buf.as_ptr() as *const c_void,
                    buf.len(),
                    NETCONN_COPY as u8,
                    &mut bytes_written,
                )
            };

            let err = NetconnErr::from(result);

            if err == NetconnErr::WouldBlock {
                futures_lite::future::yield_now().await;
                continue;
            }

            if err != NetconnErr::Ok {
                return Err(err);
            }

            break;
        }

        Ok(bytes_written)
    }
}

impl embedded_io_async::Read for NetconnTcp {
    /// This is an unoptimized way of reading that copies data to the given buffer.
    /// If there's more data available than it fits in a given buf, it's discarded.
    ///
    /// ## Use read_no_copy() instead of this method for large transfers
    ///
    /// API: `TCP`
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut pbuf = self.read_no_copy_async().await?;
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