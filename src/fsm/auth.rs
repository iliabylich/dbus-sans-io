use crate::{fsm::ReadBuffer, types::GUID};
use anyhow::{Result, bail, ensure};

#[derive(Debug)]
pub enum AuthFSM {
    WritingZero,
    WritingAuthExternal { written: usize },
    ReadingData { buf: ReadBuffer },
    WritingData { written: usize },
    ReadingGUID { buf: ReadBuffer },
    WritingBegin { written: usize, buf: Vec<u8> },
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AuthWants<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
}

impl AuthFSM {
    pub fn new() -> Self {
        Self::WritingZero
    }

    pub fn wants(&mut self) -> AuthWants<'_> {
        match self {
            Self::WritingZero => AuthWants::Write(b"\0"),
            Self::WritingAuthExternal { written } => {
                let rem = &AUTH_EXTERNAL[*written..];
                AuthWants::Write(rem)
            }
            Self::ReadingData { buf } => AuthWants::Read(buf.remaining_part_mut()),
            Self::WritingData { written, .. } => {
                let rem = &DATA[*written..];
                AuthWants::Write(rem)
            }
            Self::ReadingGUID { buf } => AuthWants::Read(buf.remaining_part_mut()),
            Self::WritingBegin { written, .. } => {
                let rem = &BEGIN[*written..];
                AuthWants::Write(rem)
            }
        }
    }

    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<()> {
        match self {
            Self::ReadingData { buf } => {
                buf.add_pos(bytes_read);
                if buf.is_full() {
                    let buf = buf.take().into_vec();
                    ensure!(buf == DATA);
                    *self = Self::WritingData { written: 0 };
                }
            }
            Self::ReadingGUID { buf } => {
                buf.add_pos(bytes_read);
                if buf.is_full() {
                    *self = Self::WritingBegin {
                        written: 0,
                        buf: buf.take().into_vec(),
                    };
                }
            }

            _ => {
                bail!("didn't expect read while in {self:?}")
            }
        }
        Ok(())
    }

    pub(crate) fn satisfy_write(&mut self, bytes_written: usize) -> Result<Option<GUID>> {
        match self {
            Self::WritingZero => {
                ensure!(bytes_written == 1);
                *self = Self::WritingAuthExternal { written: 0 };
                Ok(None)
            }
            Self::WritingAuthExternal { written } => {
                *written += bytes_written;
                ensure!(*written <= AUTH_EXTERNAL.len());
                if *written == AUTH_EXTERNAL.len() {
                    *self = Self::ReadingData {
                        buf: ReadBuffer::new(DATA.len()),
                    };
                }
                Ok(None)
            }
            Self::WritingData { written } => {
                *written += bytes_written;
                ensure!(*written <= DATA.len());
                if *written == DATA.len() {
                    *self = Self::ReadingGUID {
                        buf: ReadBuffer::new(GUID::LENGTH),
                    };
                }
                Ok(None)
            }
            Self::WritingBegin { written, buf } => {
                *written += bytes_written;
                ensure!(*written <= BEGIN.len());
                if *written == BEGIN.len() {
                    let buf = std::mem::take(buf);
                    let guid = GUID::try_from(buf)?;
                    Ok(Some(guid))
                } else {
                    Ok(None)
                }
            }

            _ => {
                bail!("didn't expect write while in {self:?}")
            }
        }
    }
}

#[test]
fn test_auth_fsm() {
    let mut fsm = AuthFSM::new();
    assert_eq!(fsm.wants(), AuthWants::Write(b"\0"));
    fsm.satisfy_write(1).unwrap();

    assert_eq!(fsm.wants(), AuthWants::Write(AUTH_EXTERNAL));
    fsm.satisfy_write(AUTH_EXTERNAL.len()).unwrap();

    let AuthWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let chunk = b"DAT";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.satisfy_read(chunk.len()).unwrap();

    let AuthWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let chunk = b"A\r\n";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.satisfy_read(chunk.len()).unwrap();

    assert_eq!(fsm.wants(), AuthWants::Write(DATA));
    fsm.satisfy_write(3).unwrap();

    assert_eq!(fsm.wants(), AuthWants::Write(b"A\r\n"));
    fsm.satisfy_write(3).unwrap();

    let AuthWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let guid = b"OK a97099b37b54cdc2a686559c6922fdeb\r\n";
    buffer.copy_from_slice(guid);
    fsm.satisfy_read(guid.len()).unwrap();

    assert_eq!(fsm.wants(), AuthWants::Write(BEGIN));
    let guid = fsm.satisfy_write(BEGIN.len()).unwrap().unwrap();

    assert_eq!(guid.as_str().unwrap(), "a97099b37b54cdc2a686559c6922fdeb");
}
