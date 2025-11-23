use crate::decoders::{Signature, Value};
use anyhow::{Result, bail, ensure};

#[derive(Default, Debug)]
pub(crate) struct HeaderFields {
    pub(crate) member: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) path: Option<Vec<u8>>,
    pub(crate) error_name: Option<String>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) signature: Option<String>,
    pub(crate) unix_fds: Option<u32>,
}

impl HeaderFields {
    pub(crate) fn new(bytes: &[u8], mut pos: usize) -> Result<Self> {
        let mut member = None;
        let mut interface = None;
        let mut path = None;
        let mut error_name = None;
        let mut reply_serial = None;
        let mut destination = None;
        let mut sender = None;
        let mut signature = None;
        let mut unix_fds = None;

        while pos < bytes.len() {
            pos = pos.next_multiple_of(8);

            let Some(header_field) = bytes.get(pos).copied() else {
                break;
            };
            pos += 1;
            let header_field = HeaderField::try_from(header_field)?;

            let Some(sig_len) = bytes.get(pos).copied() else {
                bail!("failed to read signature length");
            };
            let sig_len = sig_len as usize;
            ensure!(sig_len == 1);
            pos += 1;

            let Some(header_signature) = bytes.get(pos..pos + sig_len) else {
                bail!("failed to read single-byte header signature");
            };
            let header_signature = Signature::parse_to_end(header_signature).unwrap();
            pos += sig_len;
            pos += 1; // skip signature null terminator

            let (value, len) = Value::read_by_signature(&bytes, pos, &header_signature)?;
            pos += len;

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

        Ok(HeaderFields {
            member,
            interface,
            path,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HeaderField {
    Invalid = 0,
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
    UnixFds = 9,
}

impl TryFrom<u8> for HeaderField {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Path),
            2 => Ok(Self::Interface),
            3 => Ok(Self::Member),
            4 => Ok(Self::ErrorName),
            5 => Ok(Self::ReplySerial),
            6 => Ok(Self::Destination),
            7 => Ok(Self::Sender),
            8 => Ok(Self::Signature),
            _ => bail!("unknown header field type {value}"),
        }
    }
}
