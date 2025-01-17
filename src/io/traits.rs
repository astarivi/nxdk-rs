use core::error::Error;
use core::fmt;
use crate::winapi::time::{sleep, Timer};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum IoWrappedErr<E> {
    FmtError,
    UnexpectedEof,
    Timeout,
    Other(E),
}

impl<E> From<E> for IoWrappedErr<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

impl<E: fmt::Debug> fmt::Display for IoWrappedErr<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<E: fmt::Debug + 'static> Error for IoWrappedErr<E> {
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),
    /// Sets the offset to the size of this object plus the specified number of bytes.
    End(i64),
    /// Sets the offset to the current position plus the specified number of bytes.
    Current(i64),
}

pub trait Read {
    type ReadError: Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::ReadError>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), IoWrappedErr<Self::ReadError>> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) => return Err(IoWrappedErr::Other(e)),
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(IoWrappedErr::UnexpectedEof)
        }
    }
}

pub trait Seek {
    type SeekError: Error;

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::SeekError>;
    fn rewind(&mut self) -> Result<(), Self::SeekError> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }
    fn stream_position(&mut self) -> Result<u64, Self::SeekError> {
        self.seek(SeekFrom::Current(0))
    }
}

pub trait Write {
    type WriteError: Error;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::WriteError>;
    fn flush(&mut self) -> Result<(), Self::WriteError>;
    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::WriteError> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => panic!("write() returned Ok(0)"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<(), IoWrappedErr<Self::WriteError>> {
        struct Adapter<'a, T: Write + ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<(), T::WriteError>,
        }

        impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };

        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => match output.error {
                Err(e) => Err(IoWrappedErr::Other(e)),
                Ok(()) => Err(IoWrappedErr::FmtError),
            },
        }
    }
}

/// A wrapper around a Write trait implementer that retries
/// the write for the given amount of time.
///
/// Useful for non-blocking facilities, or for creating
/// failure resistant structures in failure prone environments.
///
/// # Example:
///
/// ```
/// use nxdk_rs::io::traits::{RetryWrite, Write};
/// use nxdk_rs::lwip::netconn::tcp::{NetconnTcp, NetconnTcpType};
/// 
/// // Where we want to write to. Ideally, connect this somewhere
/// let write_to = NetconnTcp::new(NetconnTcpType::Tcp)?;
///
/// // Data to write
/// let buf: [u8; 100] = [0; 100];
/// 
/// let mut retry = RetryWrite::new(write_to, 60, 200);
///
/// // Blocks until written, or timed out
/// let bytes_written = retry.write(&buf)?;
/// ```
pub struct RetryWrite<W: Write> {
    timeout: u64,
    sleep_for: u32,
    inner: W
}

impl<W: Write> RetryWrite<W> {
    /// Creates a new wrapper.
    ///
    /// # Arguments
    ///
    /// - `w` is the Write trait implementer.
    /// - `timeout_secs` is the amount of time to try to write for. In seconds.
    /// - `sleep_between` is the amount of time to sleep between tries. In milliseconds.
    pub fn new(w: W, timeout_secs: u64, sleep_between: u32) -> Self {
        Self { timeout: timeout_secs, inner: w, sleep_for: sleep_between }
    }
}

impl<W: Write> Write for RetryWrite<W>
where
    W: Write,
    W::WriteError: 'static,
{
    type WriteError = IoWrappedErr<<W as Write>::WriteError>;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::WriteError> {
        let start_time = Timer::new();

        loop {
            if start_time.elapsed().as_secs() >= self.timeout {
                return Err(Self::WriteError::Timeout);
            }

            let res = self.inner.write(buf).map_err(|e| IoWrappedErr::Other(e))?;

            if res != 0 {
                return Ok(res)
            }

            sleep(self.sleep_for);
        }
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        // Not implemented. Why would you want to flush like this anyway?
        unimplemented!()
    }
}
