use anyhow::{Result, bail};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HeaderField {
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
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
