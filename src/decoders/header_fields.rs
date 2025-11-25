use crate::{
    decoders::{DecodingBuffer, SignatureDecoder, ValueDecoder},
    types::{HeaderField, ObjectPath, Value},
};
use anyhow::{Result, bail, ensure};

#[derive(Default, Debug)]
pub(crate) struct HeaderFieldsDecoder {
    pub(crate) path: Option<ObjectPath>,
    pub(crate) interface: Option<String>,
    pub(crate) member: Option<String>,
    pub(crate) error_name: Option<String>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) signature: Option<Vec<u8>>,
    pub(crate) unix_fds: Option<u32>,
}

impl HeaderFieldsDecoder {
    pub(crate) fn new(mut buf: DecodingBuffer) -> Result<Self> {
        let mut path = None;
        let mut interface = None;
        let mut member = None;
        let mut error_name = None;
        let mut reply_serial = None;
        let mut destination = None;
        let mut sender = None;
        let mut signature = None;
        let mut unix_fds = None;

        while !buf.is_eof() {
            let (header_field, value) = read_header(&mut buf)?;

            match (header_field, value) {
                (HeaderField::Invalid, value) => {
                    bail!("got header Invalid with value {value:?}");
                }
                (HeaderField::Path, Value::ObjectPath(value)) => {
                    path = Some(value);
                }
                (HeaderField::Interface, Value::String(value)) => {
                    interface = Some(value);
                }
                (HeaderField::Member, Value::String(value)) => {
                    member = Some(value);
                }
                (HeaderField::ErrorName, Value::String(value)) => {
                    error_name = Some(value);
                }
                (HeaderField::ReplySerial, Value::UInt32(value)) => {
                    reply_serial = Some(value);
                }
                (HeaderField::Destination, Value::String(value)) => {
                    destination = Some(value);
                }
                (HeaderField::Sender, Value::String(value)) => {
                    sender = Some(value);
                }
                (HeaderField::Signature, Value::Signature(value)) => {
                    signature = Some(value);
                }
                (HeaderField::UnixFds, Value::UInt32(value)) => {
                    unix_fds = Some(value);
                }
                (sig, value) => {
                    bail!(
                        "invalid combination of header field signature/value: {sig:?} vs {value:?}"
                    );
                }
            }
        }

        Ok(HeaderFieldsDecoder {
            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        })
    }
}

fn read_header(buf: &mut DecodingBuffer<'_>) -> Result<(HeaderField, Value)> {
    buf.align(8)?;

    let header_field = HeaderField::from(buf.next_u8()?);

    let signature = {
        let content = ValueDecoder::decode_signature(buf)?;
        let mut buf = DecodingBuffer::new(&content);
        let signature = SignatureDecoder::parse(&mut buf)?;
        ensure!(buf.is_eof());
        signature
    };

    let value = ValueDecoder::decode_value(buf, &signature)?;

    Ok((header_field, value))
}
