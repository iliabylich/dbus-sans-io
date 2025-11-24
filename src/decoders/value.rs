use crate::{
    decoders::DecodingBuffer,
    encoders::EncodingBuffer,
    types::{Signature, Value},
};
use anyhow::Result;

pub(crate) struct ValueDecoder;

impl ValueDecoder {
    fn decode_u8(buffer: &mut DecodingBuffer) -> Result<u8> {
        buffer.next_u8()
    }

    fn decode_bool(buf: &mut DecodingBuffer) -> Result<bool> {
        Self::decode_u32(buf).map(|v| v != 0)
    }

    fn decode_i16(buf: &mut DecodingBuffer) -> Result<i16> {
        buf.align(2)?;
        buf.next_i16()
    }

    fn decode_u16(buf: &mut DecodingBuffer) -> Result<u16> {
        buf.align(2)?;
        buf.next_u16()
    }

    fn decode_i32(buf: &mut DecodingBuffer) -> Result<i32> {
        buf.align(4)?;
        buf.next_i32()
    }

    fn decode_u32(buf: &mut DecodingBuffer) -> Result<u32> {
        buf.align(4)?;
        buf.next_u32()
    }

    fn decode_i64(buf: &mut DecodingBuffer) -> Result<i64> {
        buf.align(8)?;
        buf.next_i64()
    }

    fn decode_u64(buf: &mut DecodingBuffer) -> Result<u64> {
        buf.align(8)?;
        buf.next_u64()
    }

    fn decode_f64(buf: &mut DecodingBuffer) -> Result<f64> {
        buf.align(8)?;
        buf.next_f64()
    }

    fn decode_string(buf: &mut DecodingBuffer) -> Result<String> {
        let len = Self::decode_u32(buf)? as usize;
        let s = String::from_utf8_lossy(buf.next_n(len)?).into_owned();
        buf.skip();
        Ok(s)
    }

    fn decode_object_path(buf: &mut DecodingBuffer) -> Result<Vec<u8>> {
        let len = Self::decode_u32(buf)? as usize;
        let bytes = buf.next_n(len)?.to_vec();
        buf.skip();
        Ok(bytes)
    }

    pub(crate) fn decode_signature(buf: &mut DecodingBuffer) -> Result<Vec<u8>> {
        let len = Self::decode_u8(buf)? as usize;
        let s = buf.next_n(len)?.to_vec();
        buf.skip();
        Ok(s)
    }

    pub(crate) fn decode_value(buf: &mut DecodingBuffer, signature: &Signature) -> Result<Value> {
        match signature {
            Signature::Byte => {
                let value = Self::decode_u8(buf)?;
                Ok(Value::Byte(value))
            }
            Signature::Bool => {
                let value = Self::decode_bool(buf)?;
                Ok(Value::Bool(value))
            }
            Signature::Int16 => {
                let value = Self::decode_i16(buf)?;
                Ok(Value::Int16(value))
            }
            Signature::UInt16 => {
                let value = Self::decode_u16(buf)?;
                Ok(Value::UInt16(value))
            }
            Signature::Int32 => {
                let value = Self::decode_i32(buf)?;
                Ok(Value::Int32(value))
            }
            Signature::UInt32 => {
                let value = Self::decode_u32(buf)?;
                Ok(Value::UInt32(value))
            }
            Signature::Int64 => {
                let value = Self::decode_i64(buf)?;
                Ok(Value::Int64(value))
            }
            Signature::UInt64 => {
                let value = Self::decode_u64(buf)?;
                Ok(Value::UInt64(value))
            }
            Signature::Double => {
                let value = Self::decode_f64(buf)?;
                Ok(Value::Double(value))
            }
            Signature::UnixFD => {
                let value = Self::decode_u32(buf)?;
                Ok(Value::UnixFD(value))
            }
            Signature::String => {
                let value = Self::decode_string(buf)?;
                Ok(Value::String(value))
            }
            Signature::ObjectPath => {
                let value = Self::decode_object_path(buf)?;
                Ok(Value::ObjectPath(value))
            }
            Signature::Signature => {
                let value = Self::decode_signature(buf)?;
                Ok(Value::Signature(value))
            }
            Signature::Struct(signatures) => {
                let mut fields = vec![];
                for signature in signatures {
                    let value = Self::decode_value(buf, signature)?;
                    fields.push(value);
                }
                Ok(Value::Struct(fields))
            }
            Signature::Array(item_signature) => {
                let items_count = Self::decode_u32(buf)?;
                let mut items = Vec::with_capacity(items_count as usize);
                for _ in 0..items_count {
                    let item = Self::decode_value(buf, item_signature)?;
                    items.push(item);
                }
                Ok(Value::Array(items))
            }
            Signature::Variant => todo!(),
        }
    }

    pub(crate) fn decode_many(
        buf: &mut DecodingBuffer,
        signatures: &[Signature],
    ) -> Result<Vec<Value>> {
        let mut out = vec![];
        for signature in signatures {
            let value = Self::decode_value(buf, &signature)?;
            out.push(value);
        }
        Ok(out)
    }
}

#[test]
fn test_read_byte() {
    let mut buf = DecodingBuffer::new(b"\xFF").with_pos(0);
    assert_eq!(ValueDecoder::decode_u8(&mut buf).unwrap(), 255);
    assert!(buf.is_eof());
}

#[test]
fn test_read_bool() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x01\x00\x00\x00").with_pos(1);
    assert_eq!(ValueDecoder::decode_bool(&mut buf).unwrap(), true);
    assert!(buf.is_eof());

    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x00\x00\x00\x00").with_pos(1);
    assert_eq!(ValueDecoder::decode_bool(&mut buf).unwrap(), false);
    assert!(buf.is_eof());
}

#[test]
fn test_read_int16() {
    let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_i16(&mut buf).unwrap(),
        0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}

#[test]
fn test_read_uint16() {
    let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_u16(&mut buf).unwrap(),
        0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}

#[test]
fn test_read_int32() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_i32(&mut buf).unwrap(),
        0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}

#[test]
fn test_read_uint32() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_u32(&mut buf).unwrap(),
        0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
    );
    assert!(buf.is_eof());
}

#[test]
fn test_read_int64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_i64(&mut buf).unwrap(),
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
fn test_read_uint64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08").with_pos(1);
    assert_eq!(
        ValueDecoder::decode_u64(&mut buf).unwrap(),
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
fn test_read_f64() {
    let mut buf =
        DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F").with_pos(1);
    assert_eq!(ValueDecoder::decode_f64(&mut buf).unwrap(), 0.123);
    assert!(buf.is_eof())
}

#[test]
fn test_read_object_path() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0").with_pos(1);
    assert_eq!(ValueDecoder::decode_object_path(&mut buf).unwrap(), b"abcd");
    assert!(buf.is_eof())
}

#[test]
fn test_read_string() {
    let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0").with_pos(1);
    assert_eq!(ValueDecoder::decode_string(&mut buf).unwrap(), "abcd");
    assert!(buf.is_eof())
}

#[test]
fn test_read_signature() {
    let mut buf = DecodingBuffer::new(b"\0\x04abcd\0").with_pos(1);
    assert_eq!(ValueDecoder::decode_signature(&mut buf).unwrap(), b"abcd");
    assert!(buf.is_eof())
}
