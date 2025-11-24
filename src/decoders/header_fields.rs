use crate::{
    decoders::{DecodingBuffer, SignatureDecoder, ValueDecoder},
    types::{HeaderField, Value},
};
use anyhow::{Result, bail, ensure};

#[derive(Default, Debug)]
pub(crate) struct HeaderFieldsDecoder {
    pub(crate) path: Option<Vec<u8>>,
    pub(crate) interface: Option<String>,
    pub(crate) member: Option<String>,
    pub(crate) error_name: Option<String>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) body_signature: Option<Vec<u8>>,
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
        let mut body_signature = None;
        let mut unix_fds = None;

        while !buf.is_eof() {
            buf.align(8)?;

            let Ok(header_field) = buf.next_u8() else {
                break;
            };
            let header_field = HeaderField::from(header_field);

            let signature = ValueDecoder::decode_signature(&mut buf)?;
            let mut signature_buf = DecodingBuffer::new(&signature);
            let signature = SignatureDecoder::parse(&mut signature_buf)?;
            ensure!(signature_buf.is_eof());

            let value = ValueDecoder::decode_value(&mut buf, &signature)?;

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
                    body_signature = Some(value);
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
            body_signature,
            unix_fds,
        })
    }
}
