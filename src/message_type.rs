use anyhow::{Result, bail};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Invalid = 0,
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

impl TryFrom<u8> for MessageType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::MethodCall),
            2 => Ok(Self::MethodReturn),
            3 => Ok(Self::Error),
            4 => Ok(Self::Signal),
            _ => bail!("unknown message type {value}"),
        }
    }
}
