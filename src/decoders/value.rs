use crate::types::{Signature, Value};
use anyhow::{Context, Result, bail};

pub trait ReadWriteValue: Sized {
    const ALIGN: usize;

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)>;
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        Ok((at!(buf, pos), 1))
    }

    fn write(self, buf: &mut Vec<u8>) {
        buf.push(self);
    }
}
#[test]
fn test_read_byte() {
    assert_eq!(u8::read(b"\xFF", 0).unwrap(), (255, 1));
}
#[test]
fn test_write_byte() {
    let mut v = vec![];
    42_u8.write(&mut v);
    assert_eq!(v, vec![42]);
}

impl ReadWriteValue for bool {
    const ALIGN: usize = u32::ALIGN;

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        u32::read(buf, pos).map(|(v, len)| (dbg!(v) != 0, len))
    }

    fn write(self, buf: &mut Vec<u8>) {
        let value = if self { 1_i32 } else { 0 };
        value.write(buf);
    }
}
#[test]
fn test_read_bool() {
    assert_eq!(
        bool::read(b"\0\0\0\0\x01\x00\x00\x00", 1).unwrap(),
        (true, 7)
    );
    assert_eq!(
        bool::read(b"\0\0\0\0\x00\x00\x00\x00", 1).unwrap(),
        (false, 7)
    )
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = i16::from_le_bytes([at!(buf, idx), at!(buf, idx + 1)]);
        idx += 2;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int16() {
    assert_eq!(
        i16::read(b"\0\0\xAA\xBB", 1).unwrap(),
        (0xBB << 8 | 0xAA, 3)
    )
}
#[test]
fn test_write_int16() {
    let mut v = vec![0];
    (0xBB_i16 << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\xAA\xBB")
}

impl ReadWriteValue for u16 {
    const ALIGN: usize = 2;

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = u16::from_le_bytes([at!(buf, idx), at!(buf, idx + 1)]);
        idx += 2;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint16() {
    assert_eq!(
        u16::read(b"\0\0\xAA\xBB", 1).unwrap(),
        (0xBB << 8 | 0xAA, 3)
    )
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = i32::from_le_bytes([
            at!(buf, idx),
            at!(buf, idx + 1),
            at!(buf, idx + 2),
            at!(buf, idx + 3),
        ]);
        idx += 4;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int32() {
    assert_eq!(
        i32::read(b"\0\0\0\0\xAA\xBB\xCC\xDD", 1).unwrap(),
        (0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA, 7)
    )
}
#[test]
fn test_write_int32() {
    let mut v = vec![0];
    (0xDD_i32 << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA).write(&mut v);
    assert_eq!(v, b"\0\0\0\0\xAA\xBB\xCC\xDD")
}

impl ReadWriteValue for u32 {
    const ALIGN: usize = 4;

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = u32::from_le_bytes([
            at!(buf, idx),
            at!(buf, idx + 1),
            at!(buf, idx + 2),
            at!(buf, idx + 3),
        ]);
        idx += 4;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint32() {
    assert_eq!(
        u32::read(b"\0\0\0\0\xAA\xBB\xCC\xDD", 1).unwrap(),
        (0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA, 7)
    )
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = i64::from_le_bytes([
            at!(buf, idx),
            at!(buf, idx + 1),
            at!(buf, idx + 2),
            at!(buf, idx + 3),
            at!(buf, idx + 4),
            at!(buf, idx + 5),
            at!(buf, idx + 6),
            at!(buf, idx + 7),
        ]);
        idx += 8;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_int64() {
    assert_eq!(
        i64::read(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08", 1).unwrap(),
        (
            0x08 << 56
                | 0x07 << 48
                | 0x06 << 40
                | 0x05 << 32
                | 0x04 << 24
                | 0x03 << 16
                | 0x02 << 8
                | 0x01,
            15
        )
    )
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = u64::from_le_bytes([
            at!(buf, idx),
            at!(buf, idx + 1),
            at!(buf, idx + 2),
            at!(buf, idx + 3),
            at!(buf, idx + 4),
            at!(buf, idx + 5),
            at!(buf, idx + 6),
            at!(buf, idx + 7),
        ]);
        idx += 8;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_uint64() {
    assert_eq!(
        u64::read(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08", 1).unwrap(),
        (
            0x08 << 56
                | 0x07 << 48
                | 0x06 << 40
                | 0x05 << 32
                | 0x04 << 24
                | 0x03 << 16
                | 0x02 << 8
                | 0x01,
            15
        )
    )
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

    fn read(buf: &[u8], pos: usize) -> Result<(Self, usize)> {
        let mut idx = align_read!(pos);
        let value = f64::from_le_bytes([
            at!(buf, idx),
            at!(buf, idx + 1),
            at!(buf, idx + 2),
            at!(buf, idx + 3),
            at!(buf, idx + 4),
            at!(buf, idx + 5),
            at!(buf, idx + 6),
            at!(buf, idx + 7),
        ]);
        idx += 8;
        Ok((value, idx - pos))
    }

    fn write(self, buf: &mut Vec<u8>) {
        align_write!(buf);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
#[test]
fn test_read_f64() {
    assert_eq!(
        f64::read(b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F", 1).unwrap(),
        (0.123, 15)
    )
}

#[test]
fn test_write_f64() {
    let mut v = vec![0];
    (0.123).write(&mut v);
    assert_eq!(v, b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F")
}

// string

fn read_string(buf: &[u8], mut pos: usize) -> Result<(String, usize)> {
    let (content_len, u32_len) = u32::read(buf, pos)?;
    pos += u32_len;
    let s = String::from_utf8_lossy(&buf[pos..pos + content_len as usize]).into_owned();
    Ok((s, u32_len + content_len as usize + 1))
}
fn write_str(s: &str, buf: &mut Vec<u8>) {
    (s.len() as u32).write(buf);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
}
#[test]
fn test_read_string() {
    assert_eq!(
        read_string(b"\0\0\0\0\x04\0\0\0abcd\0", 1).unwrap(),
        ("abcd".to_string(), 12)
    )
}
#[test]
fn test_write_string() {
    let mut v = vec![0];
    write_str("abcd", &mut v);
    assert_eq!(v, b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

// object path

fn read_object_path(buf: &[u8], mut pos: usize) -> Result<(Vec<u8>, usize)> {
    let (content_len, u32_len) = u32::read(buf, pos)?;
    pos += u32_len;
    let s = buf[pos..pos + content_len as usize].to_vec();
    Ok((s, u32_len + content_len as usize + 1))
}
fn write_object_path(path: &[u8], buf: &mut Vec<u8>) {
    (path.len() as u32).write(buf);
    buf.extend_from_slice(path);
    buf.push(0);
}
#[test]
fn test_read_object_path() {
    assert_eq!(
        read_object_path(b"\0\0\0\0\x04\0\0\0abcd\0", 1).unwrap(),
        (b"abcd".to_vec(), 12)
    )
}
#[test]
fn test_write_object_path() {
    let mut v = vec![0];
    write_object_path(b"abcd", &mut v);
    assert_eq!(v, b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

// signature

fn read_signature(buf: &[u8], mut pos: usize) -> Result<(String, usize)> {
    let (content_len, u8_len) = u8::read(buf, pos)?;
    pos += u8_len;
    let s = String::from_utf8_lossy(&buf[pos..pos + content_len as usize]).into_owned();
    Ok((s, u8_len + content_len as usize + 1))
}
fn write_signature(sig: &str, buf: &mut Vec<u8>) {
    (sig.len() as u8).write(buf);
    buf.extend_from_slice(sig.as_bytes());
    buf.push(0);
}
#[test]
fn test_read_signature() {
    assert_eq!(
        read_signature(b"\0\x04abcd\0", 1).unwrap(),
        ("abcd".to_string(), 6)
    )
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
    pub fn read_by_signature(
        buf: &[u8],
        pos: usize,
        signature: &Signature,
    ) -> Result<(Value, usize)> {
        match signature {
            Signature::Byte => {
                let (value, len) = u8::read(buf, pos)?;
                Ok((Value::Byte(value), len))
            }
            Signature::Bool => {
                let (value, len) = bool::read(buf, pos)?;
                Ok((Value::Bool(value), len))
            }
            Signature::Int16 => {
                let (value, len) = i16::read(buf, pos)?;
                Ok((Value::Int16(value), len))
            }
            Signature::UInt16 => {
                let (value, len) = u16::read(buf, pos)?;
                Ok((Value::UInt16(value), len))
            }
            Signature::Int32 => {
                let (value, len) = i32::read(buf, pos)?;
                Ok((Value::Int32(value), len))
            }
            Signature::UInt32 => {
                let (value, len) = u32::read(buf, pos)?;
                Ok((Value::UInt32(value), len))
            }
            Signature::Int64 => {
                let (value, len) = i64::read(buf, pos)?;
                Ok((Value::Int64(value), len))
            }
            Signature::UInt64 => {
                let (value, len) = u64::read(buf, pos)?;
                Ok((Value::UInt64(value), len))
            }
            Signature::Double => {
                let (value, len) = f64::read(buf, pos)?;
                Ok((Value::Double(value), len))
            }
            Signature::UnixFD => {
                let (value, len) = u32::read(buf, pos)?;
                Ok((Value::UnixFD(value), len))
            }
            Signature::String => {
                let (value, len) = read_string(buf, pos)?;
                Ok((Value::String(value), len))
            }
            Signature::ObjectPath => {
                let (value, len) = read_object_path(buf, pos)?;
                Ok((Value::ObjectPath(value), len))
            }
            Signature::Signature => {
                let (value, len) = read_signature(buf, pos)?;
                Ok((Value::Signature(value), len))
            }
            Signature::Struct(signatures) => {
                let mut fields = vec![];
                let mut total_len = 0;
                for signature in signatures {
                    let (value, len) = Self::read_by_signature(buf, pos + total_len, signature)?;
                    fields.push(value);
                    total_len += len;
                }
                Ok((Value::Struct(fields), total_len))
            }
            Signature::Array(item_signature) => {
                let (items_count, mut total_len) = u32::read(buf, pos)?;
                let mut items = Vec::with_capacity(items_count as usize);
                for _ in 0..items_count {
                    let (item, item_len) =
                        Self::read_by_signature(buf, pos + total_len, item_signature)?;
                    items.push(item);
                    total_len += item_len;
                }
                Ok((Value::Array(items), total_len))
            }
            Signature::Variant => todo!(),
        }
    }

    pub(crate) fn read_multi(
        buf: &[u8],
        pos: usize,
        signatures: &[Signature],
    ) -> Result<(Vec<Value>, usize)> {
        let mut out = vec![];
        let mut total_len = 0;
        for signature in signatures {
            let (value, len) = Self::read_by_signature(buf, pos + total_len, &signature)?;
            out.push(value);
            total_len += len;
        }
        Ok((out, total_len))
    }
}
