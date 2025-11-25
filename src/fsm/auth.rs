use crate::{GUID, fsm::ReadBuffer};
use anyhow::{Result, bail, ensure};

#[derive(Debug)]
pub enum AuthFSM {
    WritingZero,
    WritingAuthExternal { written: usize },
    ReadingData { buf: ReadBuffer },
    WritingData { written: usize },
    ReadingGUID { buf: ReadBuffer },
    WritingBegin { written: usize, guid: GUID },
    Done { guid: GUID },
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const GUID_LENGTH: usize = 37;
const BEGIN: &[u8] = b"BEGIN\r\n";

#[derive(Debug, PartialEq, Eq)]
pub enum AuthNextAction<'a> {
    Write(&'a [u8]),
    Read(&'a mut [u8]),
    Done(GUID),
}

impl AuthFSM {
    pub fn new() -> Self {
        Self::WritingZero
    }

    pub fn next_action(&mut self) -> AuthNextAction<'_> {
        match self {
            Self::WritingZero => AuthNextAction::Write(b"\0"),
            Self::WritingAuthExternal { written } => {
                let rem = &AUTH_EXTERNAL[*written..];
                AuthNextAction::Write(rem)
            }
            Self::ReadingData { buf } => AuthNextAction::Read(buf.remaining_part()),
            Self::WritingData { written, .. } => {
                let rem = &DATA[*written..];
                AuthNextAction::Write(rem)
            }
            Self::ReadingGUID { buf } => AuthNextAction::Read(buf.remaining_part()),
            Self::WritingBegin { written, .. } => {
                let rem = &BEGIN[*written..];
                AuthNextAction::Write(rem)
            }
            Self::Done { guid } => AuthNextAction::Done(std::mem::take(guid)),
        }
    }

    pub fn done_writing(&mut self, len: usize) -> Result<()> {
        match self {
            Self::WritingZero => {
                ensure!(len == 1);
                *self = Self::WritingAuthExternal { written: 0 };
                Ok(())
            }
            Self::WritingAuthExternal { written } => {
                *written += len;
                ensure!(*written <= AUTH_EXTERNAL.len());
                if *written == AUTH_EXTERNAL.len() {
                    *self = Self::ReadingData {
                        buf: ReadBuffer::new(DATA.len()),
                    };
                }
                Ok(())
            }
            Self::ReadingData { .. } => {
                bail!("malformed state, you were supposed to READ, not WRITE (in {self:?})");
            }
            Self::WritingData { written } => {
                *written += len;
                ensure!(*written <= DATA.len());
                if *written == DATA.len() {
                    *self = Self::ReadingGUID {
                        buf: ReadBuffer::new(GUID_LENGTH),
                    };
                }
                Ok(())
            }
            Self::ReadingGUID { .. } => {
                bail!("malformed state, you were supposed to READ, not WRITE (in {self:?})");
            }
            Self::WritingBegin { written, guid } => {
                *written += len;
                ensure!(*written <= BEGIN.len());
                if *written == BEGIN.len() {
                    *self = Self::Done {
                        guid: std::mem::take(guid),
                    }
                }
                Ok(())
            }
            Self::Done { .. } => {
                bail!("malformed state, FSM is DONE (in {self:?})");
            }
        }
    }

    pub fn done_reading(&mut self, len: usize) -> Result<()> {
        match self {
            Self::WritingZero => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::WritingAuthExternal { .. } => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::ReadingData { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    let buf = buf.take().unwrap();
                    ensure!(buf == DATA);
                    *self = Self::WritingData { written: 0 };
                }
                Ok(())
            }
            Self::WritingData { .. } => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::ReadingGUID { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    let guid = GUID::try_from(buf.take().unwrap())?;
                    *self = Self::WritingBegin { written: 0, guid };
                }
                Ok(())
            }
            Self::WritingBegin { .. } => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::Done { .. } => {
                bail!("malformed state, FSM is DONE (in {self:?})");
            }
        }
    }
}

#[test]
fn test_auth_fsm() {
    let mut fsm = AuthFSM::new();
    assert_eq!(fsm.next_action(), AuthNextAction::Write(b"\0"));
    fsm.done_writing(1).unwrap();

    assert_eq!(fsm.next_action(), AuthNextAction::Write(AUTH_EXTERNAL));
    fsm.done_writing(AUTH_EXTERNAL.len()).unwrap();

    let AuthNextAction::Read(buffer) = dbg!(fsm.next_action()) else {
        panic!("wrong next action");
    };
    let chunk = b"DAT";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.done_reading(chunk.len()).unwrap();

    let AuthNextAction::Read(buffer) = dbg!(fsm.next_action()) else {
        panic!("wrong next action");
    };
    let chunk = b"A\r\n";
    buffer[..chunk.len()].copy_from_slice(chunk);
    fsm.done_reading(chunk.len()).unwrap();

    assert_eq!(fsm.next_action(), AuthNextAction::Write(DATA));
    fsm.done_writing(3).unwrap();

    assert_eq!(fsm.next_action(), AuthNextAction::Write(b"A\r\n"));
    fsm.done_writing(3).unwrap();

    let AuthNextAction::Read(buffer) = dbg!(fsm.next_action()) else {
        panic!("wrong next action");
    };
    let guid = b"OK a97099b37b54cdc2a686559c6922fdeb\r\n";
    buffer.copy_from_slice(guid);
    fsm.done_reading(guid.len()).unwrap();

    assert_eq!(fsm.next_action(), AuthNextAction::Write(BEGIN));
    fsm.done_writing(BEGIN.len()).unwrap();

    let AuthNextAction::Done(guid) = dbg!(fsm.next_action()) else {
        panic!("wrong next action");
    };
    assert_eq!(guid.as_str().unwrap(), "a97099b37b54cdc2a686559c6922fdeb");
}
