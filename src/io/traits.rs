use core::error::Error;
use core::fmt;
use crate::utils::error::PlatformError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum IoWrappedErr<E> {
    FmtError,
    UnexpectedEof,
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
