use anyhow::{Result, bail};

pub struct Flags {
    byte: u8,
}

impl TryFrom<u8> for Flags {
    type Error = anyhow::Error;

    fn try_from(byte: u8) -> Result<Self> {
        match byte {
            0..=7 => Ok(Self { byte }),
            _ => bail!("flags must be in 0..=7 range"),
        }
    }
}

impl Flags {
    pub const NO_REPLY_EXPECTED: u8 = 0x1;
    // pub const NO_REPLY_EXPECTED: u8 = 0x1;
}
