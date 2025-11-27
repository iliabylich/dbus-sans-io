use crate::{
    fsm::{FSMSatisfy, FSMWants, ReadBuffer},
    types::GUID,
};
use anyhow::{Result, ensure};

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

impl AuthFSM {
    pub fn new() -> Self {
        Self::WritingZero
    }

    pub fn wants(&mut self) -> FSMWants<'_> {
        match self {
            Self::WritingZero => FSMWants::Write(b"\0"),
            Self::WritingAuthExternal { written } => {
                let rem = &AUTH_EXTERNAL[*written..];
                FSMWants::Write(rem)
            }
            Self::ReadingData { buf } => FSMWants::Read(buf.remaining_part_mut()),
            Self::WritingData { written, .. } => {
                let rem = &DATA[*written..];
                FSMWants::Write(rem)
            }
            Self::ReadingGUID { buf } => FSMWants::Read(buf.remaining_part_mut()),
            Self::WritingBegin { written, .. } => {
                let rem = &BEGIN[*written..];
                FSMWants::Write(rem)
            }
        }
    }

    pub(crate) fn satisfy(&mut self, with: FSMSatisfy) -> Result<Option<GUID>> {
        match self {
            Self::WritingZero => {
                let len = with.require_write()?;
                ensure!(len == 1);
                *self = Self::WritingAuthExternal { written: 0 };
                Ok(None)
            }
            Self::WritingAuthExternal { written } => {
                let len = with.require_write()?;
                *written += len;
                ensure!(*written <= AUTH_EXTERNAL.len());
                if *written == AUTH_EXTERNAL.len() {
                    *self = Self::ReadingData {
                        buf: ReadBuffer::new(DATA.len()),
                    };
                }
                Ok(None)
            }
            Self::ReadingData { buf } => {
                let len = with.require_read()?;
                buf.add_pos(len);
                if buf.is_full() {
                    let buf = buf.take().into_vec();
                    ensure!(buf == DATA);
                    *self = Self::WritingData { written: 0 };
                }
                Ok(None)
            }
            Self::WritingData { written } => {
                let len = with.require_write()?;
                *written += len;
                ensure!(*written <= DATA.len());
                if *written == DATA.len() {
                    *self = Self::ReadingGUID {
                        buf: ReadBuffer::new(GUID::LENGTH),
                    };
                }
                Ok(None)
            }
            Self::ReadingGUID { buf } => {
                let len = with.require_read()?;
                buf.add_pos(len);
                if buf.is_full() {
                    *self = Self::WritingBegin {
                        written: 0,
                        buf: buf.take().into_vec(),
                    };
                }
                Ok(None)
            }
            Self::WritingBegin { written, buf } => {
                let len = with.require_write()?;
                *written += len;
                ensure!(*written <= BEGIN.len());
                if *written == BEGIN.len() {
                    let buf = std::mem::take(buf);
                    let guid = GUID::try_from(buf)?;
                    Ok(Some(guid))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[test]
fn test_auth_fsm() {
    let mut fsm = AuthFSM::new();
    assert_eq!(fsm.wants(), FSMWants::Write(b"\0"));
    fsm.satisfy(FSMSatisfy::Write { len: 1 }).unwrap();

    assert_eq!(fsm.wants(), FSMWants::Write(AUTH_EXTERNAL));
    fsm.satisfy(FSMSatisfy::Write {
        len: AUTH_EXTERNAL.len(),
    })
    .unwrap();

    let FSMWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let chunk = b"DAT";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.satisfy(FSMSatisfy::Read { len: chunk.len() }).unwrap();

    let FSMWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let chunk = b"A\r\n";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.satisfy(FSMSatisfy::Read { len: chunk.len() }).unwrap();

    assert_eq!(fsm.wants(), FSMWants::Write(DATA));
    fsm.satisfy(FSMSatisfy::Write { len: 3 }).unwrap();

    assert_eq!(fsm.wants(), FSMWants::Write(b"A\r\n"));
    fsm.satisfy(FSMSatisfy::Write { len: 3 }).unwrap();

    let FSMWants::Read(buffer) = dbg!(fsm.wants()) else {
        panic!("wrong next action");
    };
    let guid = b"OK a97099b37b54cdc2a686559c6922fdeb\r\n";
    buffer.copy_from_slice(guid);
    fsm.satisfy(FSMSatisfy::Read { len: guid.len() }).unwrap();

    assert_eq!(fsm.wants(), FSMWants::Write(BEGIN));
    let guid = fsm
        .satisfy(FSMSatisfy::Write { len: BEGIN.len() })
        .unwrap()
        .unwrap();

    assert_eq!(guid.as_str().unwrap(), "a97099b37b54cdc2a686559c6922fdeb");
}
