use crate::{
    encoders::{EncodingBuffer, SignatureEncoder},
    types::{HeaderFieldName, Value, ValueRef},
};

pub(crate) struct ValueEncoder;

impl ValueEncoder {
    pub(crate) fn encode_u8(buf: &mut EncodingBuffer, value: u8) {
        buf.encode_u8(value);
    }

    pub(crate) fn encode_bool(buf: &mut EncodingBuffer, value: bool) {
        Self::encode_u32(buf, if value { 1_u32 } else { 0 });
    }

    pub(crate) fn encode_u16(buf: &mut EncodingBuffer, value: u16) {
        buf.align(2);
        buf.encode_u16(value);
    }

    pub(crate) fn encode_i16(buf: &mut EncodingBuffer, value: i16) {
        buf.align(2);
        buf.encode_i16(value);
    }

    pub(crate) fn encode_u32(buf: &mut EncodingBuffer, value: u32) {
        buf.align(4);
        buf.encode_u32(value);
    }

    pub(crate) fn encode_i32(buf: &mut EncodingBuffer, value: i32) {
        buf.align(4);
        buf.encode_i32(value);
    }

    pub(crate) fn encode_u64(buf: &mut EncodingBuffer, value: u64) {
        buf.align(8);
        buf.encode_u64(value);
    }

    pub(crate) fn encode_i64(buf: &mut EncodingBuffer, value: i64) {
        buf.align(8);
        buf.encode_i64(value);
    }

    pub(crate) fn encode_f64(buf: &mut EncodingBuffer, value: f64) {
        buf.align(8);
        buf.encode_f64(value);
    }

    pub(crate) fn encode_str(buf: &mut EncodingBuffer, s: &str) {
        Self::encode_u32(buf, s.len() as u32);
        buf.encode_bytes(s.as_bytes());
        buf.encode_u8(0);
    }

    pub(crate) fn encode_object_path(buf: &mut EncodingBuffer, path: &[u8]) {
        Self::encode_u32(buf, path.len() as u32);
        buf.encode_bytes(path);
        buf.encode_u8(0);
    }

    pub(crate) fn encode_signature(buf: &mut EncodingBuffer, sig: &[u8]) {
        Self::encode_u8(buf, sig.len() as u8);
        buf.encode_bytes(sig);
        buf.encode_u8(0);
    }

    pub(crate) fn encode_struct(buf: &mut EncodingBuffer, fields: &[Value]) {
        Self::encode_u32(buf, fields.len() as u32);
        for field in fields {
            Self::encode_value(buf, ValueRef::from(field));
        }
    }

    pub(crate) fn encode_array(buf: &mut EncodingBuffer, items: &[Value]) {
        Self::encode_u32(buf, items.len() as u32);
        for item in items {
            Self::encode_value(buf, ValueRef::from(item));
        }
    }

    pub(crate) fn encode_header(buf: &mut EncodingBuffer, field: HeaderFieldName, value: &Value) {
        buf.encode_u8(field as u8);
        buf.encode_u8(0);
        let start = buf.size();
        SignatureEncoder::encode_complete_type(buf, &value.complete_type());
        buf.set_u8(start - 1, (buf.size() - start) as u8).unwrap();
        buf.encode_u8(0);
        Self::encode_value(buf, ValueRef::from(value));
    }

    pub(crate) fn encode_value(buf: &mut EncodingBuffer, value: ValueRef<'_>) {
        match value {
            ValueRef::Byte(value) => Self::encode_u8(buf, value),
            ValueRef::Bool(value) => Self::encode_bool(buf, value),
            ValueRef::Int16(value) => Self::encode_i16(buf, value),
            ValueRef::UInt16(value) => Self::encode_u16(buf, value),
            ValueRef::Int32(value) => Self::encode_i32(buf, value),
            ValueRef::UInt32(value) => Self::encode_u32(buf, value),
            ValueRef::Int64(value) => Self::encode_i64(buf, value),
            ValueRef::UInt64(value) => Self::encode_u64(buf, value),
            ValueRef::Double(value) => Self::encode_f64(buf, value),
            ValueRef::UnixFD(value) => Self::encode_u32(buf, value),
            ValueRef::String(s) => Self::encode_str(buf, s),
            ValueRef::ObjectPath(path) => Self::encode_object_path(buf, path),
            ValueRef::Signature(sig) => Self::encode_signature(buf, sig),
            ValueRef::Struct(fields) => Self::encode_struct(buf, fields),
            ValueRef::Array(_item_type, items) => Self::encode_array(buf, items),
            ValueRef::Variant(_inner) => todo!(),
        }
    }
}

#[test]
fn test_encode_byte() {
    let mut buf = EncodingBuffer::new();
    ValueEncoder::encode_u8(&mut buf, 42);
    assert_eq!(buf.done(), vec![42]);
}

#[test]
fn test_encode_bool() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_bool(&mut buf, true);
    assert_eq!(buf.done(), b"\0\0\0\0\x01\x00\x00\x00");

    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_bool(&mut buf, false);
    assert_eq!(buf.done(), b"\0\0\0\0\x00\x00\x00\x00");
}

#[test]
fn test_encode_int16() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_i16(&mut buf, 0xBB << 8 | 0xAA);
    assert_eq!(buf.done(), b"\0\0\xAA\xBB")
}

#[test]
fn test_encode_uint16() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_u16(&mut buf, 0xBB << 8 | 0xAA);
    assert_eq!(buf.done(), b"\0\0\xAA\xBB")
}

#[test]
fn test_encode_int32() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_i32(&mut buf, 0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA);
    assert_eq!(buf.done(), b"\0\0\0\0\xAA\xBB\xCC\xDD")
}

#[test]
fn test_encode_uint32() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_u32(&mut buf, 0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA);
    assert_eq!(buf.done(), b"\0\0\0\0\xAA\xBB\xCC\xDD")
}

#[test]
fn test_encode_int64() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_i64(
        &mut buf,
        0x08 << 56
            | 0x07 << 48
            | 0x06 << 40
            | 0x05 << 32
            | 0x04 << 24
            | 0x03 << 16
            | 0x02 << 8
            | 0x01,
    );
    assert_eq!(
        buf.done(),
        b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08"
    )
}

#[test]
fn test_encode_uint64() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_u64(
        &mut buf,
        0x08_u64 << 56
            | 0x07 << 48
            | 0x06 << 40
            | 0x05 << 32
            | 0x04 << 24
            | 0x03 << 16
            | 0x02 << 8
            | 0x01,
    );
    assert_eq!(
        buf.done(),
        b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08"
    )
}

#[test]
fn test_encode_f64() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_f64(&mut buf, 0.123);
    assert_eq!(
        buf.done(),
        b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F"
    )
}

#[test]
fn test_encode_string() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_str(&mut buf, "abcd");
    assert_eq!(buf.done(), b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

#[test]
fn test_encode_object_path() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);
    ValueEncoder::encode_object_path(&mut buf, b"abcd");
    assert_eq!(buf.done(), b"\0\0\0\0\x04\x00\x00\x00abcd\0")
}

#[test]
fn test_encode_signature() {
    let mut buf = EncodingBuffer::new();
    buf.encode_u8(0);

    ValueEncoder::encode_signature(&mut buf, b"abcd");
    assert_eq!(buf.done(), b"\0\x04abcd\0")
}
