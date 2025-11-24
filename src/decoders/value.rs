use crate::{
    decoders::DecodingBuffer,
    types::{Signature, Value},
};
use anyhow::Result;

pub trait ReadWriteValue: Sized {
    const ALIGN: usize;

    fn read(buf: &mut DecodingBuffer) -> Result<Self>;
    fn write(self, buf: &mut Vec<u8>);
}
macro_rules! at {
    ($buf:expr, $pos:expr) => {
        $buf.get($pos).copied().context("EOF")?
    };
}
macro_rules! align_read {
    ($pos:expr) => {
        $pos.next_multiple_of(Self::ALIGN)
    };
}
macro_rules! align_write {
    ($buf:expr, $align:expr) => {
        while $buf.len() % $align != 0 {
            $buf.push(0);
        }
    };
    ($buf:expr) => {
        align_write!($buf, Self::ALIGN)
    };
}

impl ReadWriteValue for u8 {
    const ALIGN: usize = 1;

    fn read(buffer: &mut DecodingBuffer) -> Result<Self> {
        buffer.next_u8()
    }

    fn write(self, buf: &mut Vec<u8>) {
        buf.push(self);
    }
}
#[test]
fn test_read_byte() {
    let mut buf = DecodingBuffer::new(b"\xFF").with_pos(0);
    assert_eq!(u8::read(&mut buf).unwrap(), 255);
    assert!(buf.is_eof());
}
#[test]
fn test_write_byte() {
    let mut v = vec![];
    42_u8.write(&mut v);
    assert_eq!(v, vec![42]);
}

impl ReadWriteValue for bool {
    const ALIGN: usize = u32::ALIGN;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        u32::read(buf).map(|v| v != 0)
    }

    fn write(self, buf: &mut Vec<u8>) {
        let value = if self { 1_i32 } else { 0 };
        value.write(buf);
    }
}
#[test]
fn test_read_bool() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x01\x00\x00\x00").with_pos(1);
    assert_eq!(bool::read(&mut buf).unwrap(), true);
    assert!(buf.is_eof());

    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x00\x00\x00\x00").with_pos(1);
    assert_eq!(bool::read(&mut buf).unwrap(), false);
    assert!(buf.is_eof());
}
#[test]
fn test_write_bool() {
    let mut v = vec![0];
    true.write(&mut v);
    assert_eq!(v, b"\0\0\0\0\x01\x00\x00\x00");

    let mut v = vec![0];
    false.write(&mut v);
    assert_eq!(v, b"\0\0\0\0\x00\x00\x00\x00");
}

// 16

impl ReadWriteValue for i16 {
    const ALIGN: usize = 2;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(2)?;
        buf.next_i16()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int16() {
    let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB").with_pos(1);
    assert_eq!(i16::read(&mut buf).unwrap(), 0xBB << 8 | 0xAA);
    assert!(buf.is_eof());
}
#[test]
fn test_write_int16() {
    let mut v = vec![0];
    (0xBB_i16 << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\xAA\xBB")
}

impl ReadWriteValue for u16 {
    const ALIGN: usize = 2;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(2)?;
        buf.next_u16()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint16() {
    let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB").with_pos(1);
    assert_eq!(u16::read(&mut buf).unwrap(), 0xBB << 8 | 0xAA);
    assert!(buf.is_eof());
}
#[test]
fn test_write_uint16() {
    let mut v = vec![0];
    (0xBB_u16 << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\xAA\xBB")
}

// 32

impl ReadWriteValue for i32 {
    const ALIGN: usize = 4;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(4)?;
        buf.next_i32()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int32() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD").with_pos(1);
    assert_eq!(
        i32::read(&mut buf).unwrap(),
        0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}
#[test]
fn test_write_int32() {
    let mut v = vec![0];
    (0xDD_i32 << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\0\0\xAA\xBB\xCC\xDD")
}

impl ReadWriteValue for u32 {
    const ALIGN: usize = 4;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(4)?;
        buf.next_u32()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint32() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD").with_pos(1);
    assert_eq!(
        u32::read(&mut buf).unwrap(),
        0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}
#[test]
fn test_write_uint32() {
    let mut v = vec![0];
    (0xDD_u32 << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\0\0\xAA\xBB\xCC\xDD")
}

// 64

impl ReadWriteValue for i64 {
    const ALIGN: usize = 8;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(8)?;
        buf.next_i64()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08").with_pos(1);
    assert_eq!(
        i64::read(&mut buf).unwrap(),
        0x08 << 56
            | 0x07 << 48
            | 0x06 << 40
            | 0x05 << 32
            | 0x04 << 24
            | 0x03 << 16
            | 0x02 << 8
            | 0x01,
    );
    assert!(buf.is_eof());
}
#[test]
fn test_write_int64() {
    let mut v = vec![0];
    (0x08_i64 << 56
        | 0x07 << 48
        | 0x06 << 40
        | 0x05 << 32
        | 0x04 << 24
        | 0x03 << 16
        | 0x02 << 8
        | 0x01)
        .write(&mut v);
    assert_eq!(v, b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08")
}

impl ReadWriteValue for u64 {
    const ALIGN: usize = 8;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(8)?;
        buf.next_u64()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08").with_pos(1);
    assert_eq!(
        u64::read(&mut buf).unwrap(),
        0x08 << 56
            | 0x07 << 48
            | 0x06 << 40
            | 0x05 << 32
            | 0x04 << 24
            | 0x03 << 16
            | 0x02 << 8
            | 0x01,
    );
    assert!(buf.is_eof());
}
#[test]
fn test_write_uint64() {
    let mut v = vec![0];
    (0x08_u64 << 56
        | 0x07 << 48
        | 0x06 << 40
        | 0x05 << 32
        | 0x04 << 24
        | 0x03 << 16
        | 0x02 << 8
        | 0x01)
        .write(&mut v);
    assert_eq!(v, b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08")
}

// f64

impl ReadWriteValue for f64 {
    const ALIGN: usize = 8;

    fn read(buf: &mut DecodingBuffer) -> Result<Self> {
        buf.align(8)?;
        buf.next_f64()
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_f64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F").with_pos(1);
    assert_eq!(f64::read(&mut buf).unwrap(), 0.123);
    assert!(buf.is_eof())
}

#[test]
fn test_write_f64() {
    let mut v = vec![0];
    (0.123).write(&mut v);
    assert_eq!(v, b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F")
}

// string

fn read_string(buf: &mut DecodingBuffer) -> Result<String> {
    let len = u32::read(buf)? as usize;
    let s = String::from_utf8_lossy(buf.next_n(len)?).into_owned();
    buf.skip();
    Ok(s)
}
fn write_str(s: &str, buf: &mut Vec<u8>) {
    (s.len() as u32).write(buf);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
}
#[test]
fn test_read_string() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0").with_pos(1);
    assert_eq!(read_string(&mut buf).unwrap(), "abcd");
    assert!(buf.is_eof())
}
#[test]
fn test_write_string() {
    let mut v = vec![0];
    write_str("abcd", &mut v);
    assert_eq!(v, b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

// object path

fn read_object_path(buf: &mut DecodingBuffer) -> Result<Vec<u8>> {
    let len = u32::read(buf)? as usize;
    let bytes = buf.next_n(len)?.to_vec();
    buf.skip();
    Ok(bytes)
}
fn write_object_path(path: &[u8], buf: &mut Vec<u8>) {
    (path.len() as u32).write(buf);
    buf.extend_from_slice(path);
    buf.push(0);
}
#[test]
fn test_read_object_path() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0").with_pos(1);
    assert_eq!(read_object_path(&mut buf).unwrap(), b"abcd");
    assert!(buf.is_eof())
}
#[test]
fn test_write_object_path() {
    let mut v = vec![0];
    write_object_path(b"abcd", &mut v);
    assert_eq!(v, b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

// signature

pub(crate) fn read_signature(buf: &mut DecodingBuffer) -> Result<String> {
    let len = u8::read(buf)? as usize;
    let s = String::from_utf8_lossy(buf.next_n(len)?).into_owned();
    buf.skip();
    Ok(s)
}
fn write_signature(sig: &str, buf: &mut Vec<u8>) {
    (sig.len() as u8).write(buf);
    buf.extend_from_slice(sig.as_bytes());
    buf.push(0);
}
#[test]
fn test_read_signature() {
    let mut buf = DecodingBuffer::new(b"\0\x04abcd\0").with_pos(1);
    assert_eq!(read_signature(&mut buf).unwrap(), "abcd");
    assert!(buf.is_eof())
}
#[test]
fn test_write_signature() {
    let mut v = vec![0];
    write_signature("abcd", &mut v);
    assert_eq!(v, b"\0\x04abcd\0")
}

// EVERYTHING

pub(crate) struct ValueDecoder;

impl ValueDecoder {
    pub fn read_by_signature(buf: &mut DecodingBuffer, signature: &Signature) -> Result<Value> {
        match signature {
            Signature::Byte => {
                let value = u8::read(buf)?;
                Ok(Value::Byte(value))
            }
            Signature::Bool => {
                let value = bool::read(buf)?;
                Ok(Value::Bool(value))
            }
            Signature::Int16 => {
                let value = i16::read(buf)?;
                Ok(Value::Int16(value))
            }
            Signature::UInt16 => {
                let value = u16::read(buf)?;
                Ok(Value::UInt16(value))
            }
            Signature::Int32 => {
                let value = i32::read(buf)?;
                Ok(Value::Int32(value))
            }
            Signature::UInt32 => {
                let value = u32::read(buf)?;
                Ok(Value::UInt32(value))
            }
            Signature::Int64 => {
                let value = i64::read(buf)?;
                Ok(Value::Int64(value))
            }
            Signature::UInt64 => {
                let value = u64::read(buf)?;
                Ok(Value::UInt64(value))
            }
            Signature::Double => {
                let value = f64::read(buf)?;
                Ok(Value::Double(value))
            }
            Signature::UnixFD => {
                let value = u32::read(buf)?;
                Ok(Value::UnixFD(value))
            }
            Signature::String => {
                let value = read_string(buf)?;
                Ok(Value::String(value))
            }
            Signature::ObjectPath => {
                let value = read_object_path(buf)?;
                Ok(Value::ObjectPath(value))
            }
            Signature::Signature => {
                let value = read_signature(buf)?;
                Ok(Value::Signature(value))
            }
            Signature::Struct(signatures) => {
                let mut fields = vec![];
                for signature in signatures {
                    let value = Self::read_by_signature(buf, signature)?;
                    fields.push(value);
                }
                Ok(Value::Struct(fields))
            }
            Signature::Array(item_signature) => {
                let items_count = u32::read(buf)?;
                let mut items = Vec::with_capacity(items_count as usize);
                for _ in 0..items_count {
                    let item = Self::read_by_signature(buf, item_signature)?;
                    items.push(item);
                }
                Ok(Value::Array(items))
            }
            Signature::Variant => todo!(),
        }
    }

    pub(crate) fn read_multi(
        buf: &mut DecodingBuffer,
        signatures: &[Signature],
    ) -> Result<Vec<Value>> {
        let mut out = vec![];
        for signature in signatures {
            let value = Self::read_by_signature(buf, &signature)?;
            out.push(value);
        }
        Ok(out)
    }
}
