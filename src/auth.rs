use crate::{FixedSizeReader, FixedSizeWriter, IoOperation, IoReader, IoRoundtrip, IoWriter};
use anyhow::{Result, bail};
use std::io::{Read, Write};

const AUTH_EXTERNAL: [u8; 15] = [
    b'A', b'U', b'T', b'H', b' ', b'E', b'X', b'T', b'E', b'R', b'N', b'A', b'L', b'\r', b'\n',
];
const DATA: [u8; 6] = [b'D', b'A', b'T', b'A', b'\r', b'\n'];
const GUID_LENGTH: usize = 37;
const BEGIN: [u8; 7] = [b'B', b'E', b'G', b'I', b'N', b'\r', b'\n'];

pub(crate) enum Auth {
    WritingAuthExternal(FixedSizeWriter<{ AUTH_EXTERNAL.len() }>),
    ReadingData(FixedSizeReader<{ DATA.len() }>),
    WritingData(FixedSizeWriter<{ DATA.len() }>),
    ReadingGuid(FixedSizeReader<GUID_LENGTH>),
    WritingBegin {
        guid: GUID,
        writer: FixedSizeWriter<{ BEGIN.len() }>,
    },
}

impl Auth {
    pub(crate) fn new() -> Self {
        Self::WritingAuthExternal(FixedSizeWriter::new(AUTH_EXTERNAL))
    }
}

impl IoRoundtrip<GUID> for Auth {
    fn continue_roundtrip(&mut self, rw: &mut (impl Read + Write)) -> Result<IoOperation<GUID>> {
        loop {
            match self {
                Self::WritingAuthExternal(w) => match w.continue_writing(rw)? {
                    IoOperation::Finished(()) => *self = Self::ReadingData(FixedSizeReader::new()),
                    IoOperation::WouldBlock => return Ok(IoOperation::WouldBlock),
                },

                Self::ReadingData(r) => match r.continue_reading(rw)? {
                    IoOperation::Finished(data) => {
                        if data != DATA {
                            bail!("got wrong bytes after AUTH EXTERNAL: {data:?}");
                        }
                        *self = Self::WritingData(FixedSizeWriter::new(DATA));
                    }
                    IoOperation::WouldBlock => return Ok(IoOperation::WouldBlock),
                },

                Self::WritingData(w) => match w.continue_writing(rw)? {
                    IoOperation::Finished(()) => *self = Self::ReadingGuid(FixedSizeReader::new()),
                    IoOperation::WouldBlock => return Ok(IoOperation::WouldBlock),
                },

                Self::ReadingGuid(r) => match r.continue_reading(rw)? {
                    IoOperation::Finished(guid) => {
                        *self = Self::WritingBegin {
                            guid: GUID(guid),
                            writer: FixedSizeWriter::new(BEGIN),
                        }
                    }
                    IoOperation::WouldBlock => return Ok(IoOperation::WouldBlock),
                },

                Self::WritingBegin { guid, writer } => match writer.continue_writing(rw)? {
                    IoOperation::Finished(()) => return Ok(IoOperation::Finished(*guid)),
                    IoOperation::WouldBlock => return Ok(IoOperation::WouldBlock),
                },
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct GUID([u8; GUID_LENGTH]);

impl GUID {
    fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0[3..GUID_LENGTH - 2])
    }
}

impl std::fmt::Debug for GUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}

impl std::fmt::Display for GUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}
