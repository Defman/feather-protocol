//! Extension traits for `Bytes` and `BytesMut` which support Minecraft types.
use bytes::buf::{Buf, BufExt, BufMut};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;
use std::ops::Deref;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not enough bytes to read")]
    NotEnoughBytes,
    #[error("Too many bytes to read")]
    ValueTooLarge,
    #[error("Go home, you're drunk?")]
    Malformed,
}

#[derive(Debug, Clone)]
pub struct Nbt<T: DeserializeOwned + Serialize> {
    pub inner: T,
}

impl<T: DeserializeOwned + Serialize> From<T> for Nbt<T> {
    fn from(nbt: T) -> Self {
        Nbt { inner: nbt }
    }
}

impl<T: DeserializeOwned + Serialize> Deref for Nbt<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct VarInt {
    inner: i32,
}

impl From<i32> for VarInt {
    fn from(n: i32) -> Self {
        VarInt { inner: n }
    }
}

impl Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct VarLong {
    pub inner: i64,
}

impl From<i64> for VarLong {
    fn from(n: i64) -> Self {
        VarLong { inner: n }
    }
}

impl Deref for VarLong {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait TryReadInto<T: Sized>: Buf {
    fn try_read(&mut self) -> Result<T, Error>;
}

pub trait TryReadFrom: Sized {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error>;
}

impl<B, T> TryReadInto<T> for B
where
    B: Buf,
    T: TryReadFrom
{
    fn try_read(&mut self) -> Result<T, Error> {
        TryReadFrom::try_read(self)
    }
}

pub trait WriteInto: Sized {
    fn write(&self, buf: &mut impl BufMut) -> usize;
}

pub trait WriteFrom<T: Sized>: BufMut {
    fn write(&mut self, value: &T) -> usize;
}

impl<T, B> WriteFrom<T> for B
where
    B: BufMut,
    T: WriteInto,
{
    fn write(&mut self, value: &T) -> usize {
        value.write(self)
    }
}

macro_rules! try_read_write {
    ($this:ident, $size:expr, $get:ident, $put:ident) => {
        impl TryReadFrom for $this {
            fn try_read(buf: &mut impl Buf) -> Result<$this, Error> {
                if buf.remaining() < $size {
                    Err(Error::NotEnoughBytes)?
                } else {
                    Ok(buf.$get())
                }
            }
        }

        impl WriteInto for $this {
            fn write(&self, buf: &mut impl BufMut) -> usize {
                buf.$put(*self);
                $size
            }
        }
    };
}

try_read_write!(u8, 1, get_u8, put_u8);
try_read_write!(i8, 1, get_i8, put_i8);

try_read_write!(u16, 2, get_u16, put_u16);
try_read_write!(i16, 2, get_i16, put_i16);

try_read_write!(u32, 4, get_u32, put_u32);
try_read_write!(i32, 4, get_i32, put_i32);

try_read_write!(u64, 8, get_u64, put_u64);
try_read_write!(i64, 8, get_i64, put_i64);

try_read_write!(f32, 4, get_f32, put_f32);
try_read_write!(f64, 8, get_f64, put_f64);

impl TryReadFrom for VarInt {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        let mut num_read = 0;
        let mut result = 0;
        loop {
            if buf.remaining() == 0 {
                return Err(Error::NotEnoughBytes)?;
            }
            let read: u8 = buf.try_read()?;
            let value = i32::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7u32 * num_read).0;

            num_read += 1;
            if num_read > 5 {
                return Err(Error::Malformed)?;
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(VarInt::from(result))
    }
}

impl WriteInto for VarInt {
    fn write(&self, buf: &mut impl BufMut) -> usize {
        let mut x = **self;
        let mut bytes_written = 0;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }
            buf.put_u8(temp);
            bytes_written += 1;
            if x == 0 {
                break;
            }
        }

        bytes_written
    }
}

impl TryReadFrom for VarLong {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        let mut num_read = 0;
        let mut result = 0;
        loop {
            if buf.remaining() == 0 {
                return Err(Error::NotEnoughBytes)?;
            }
            let read: u8 = buf.try_read()?;
            let value = i64::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7u32 * num_read).0;

            num_read += 1;
            if num_read > 10 {
                return Err(Error::Malformed)?;
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(VarLong::from(result))
    }
}

impl TryReadFrom for String {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        let len: VarInt = buf.try_read()?;
        let len = *len as usize;
        // Check that the client isn't trying
        // to make the server allocate ridiculous
        // amounts of memory
        if len > 32767 {
            return Err(Error::ValueTooLarge)?;
        }
        let mut result = String::with_capacity(len);
        let read_len = buf
            .take(len)
            .reader()
            .read_to_string(&mut result)
            .map_err(|_| Error::Malformed)?;
        // Verify the number of bytes actually read.
        if read_len == len {
            Ok(result)
        } else {
            Err(Error::NotEnoughBytes)?
        }
    }
}

impl TryReadFrom for Uuid {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        let mut bytes = [0u8; 16];
        buf.copy_to_slice(&mut bytes);
        Ok(Uuid::from_bytes(bytes))
    }
}

impl TryReadFrom for bool {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        let val: u8 = buf.try_read()?;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Malformed)?,
        }
    }
}

impl<T: DeserializeOwned + Serialize> TryReadFrom for Nbt<T> {
    fn try_read(buf: &mut impl Buf) -> Result<Self, Error> {
        nbt::from_reader::<_, T>(buf.reader())
            .map(Nbt::from)
            .map_err(|_| Error::Malformed.into())
    }
}
