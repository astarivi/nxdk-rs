use nxdk_sys::lwip::{pbuf, pbuf_free};

/// A no-copy, TCP Pbuf implementation
pub struct TcpPbuf {
    pbuf: Option<*mut pbuf>,
    current_pbuf: Option<*mut pbuf>,
}

impl TcpPbuf {
    pub fn new(pbuf: *mut pbuf) -> Self {
        TcpPbuf { pbuf: Some(pbuf), current_pbuf: None }
    }

    pub fn close(&mut self) {
        if let Some(pbuf_ptr) = self.pbuf.take() {
            unsafe {
                pbuf_free(pbuf_ptr);
            }
        }
    }
}

impl Iterator for TcpPbuf {
    type Item = &'static [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pbuf.is_none() {
            self.current_pbuf = self.pbuf;
        }

        unsafe {
            while !self.current_pbuf?.is_null() {
                let len = (*self.current_pbuf?).len as usize;

                let buffer = core::slice::from_raw_parts(
                    (*self.current_pbuf?).payload as *const u8,
                    len
                );

                self.current_pbuf = Some((*self.current_pbuf?).next);

                return Some(buffer);
            }
        }

        self.close();
        None
    }
}
