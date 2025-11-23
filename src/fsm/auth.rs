use crate::guid::GUID;
use anyhow::{Result, bail, ensure};

#[derive(Debug)]
pub enum AuthFSM {
    WritingZero,
    WritingAuthExternal { written: usize },
    ReadingData { read: usize, buf: [u8; DATA.len()] },
    WritingData { written: usize },
    ReadingGUID { read: usize, buf: [u8; GUID_LENGTH] },
    WritingBegin { written: usize, guid: GUID },
    Done { guid: GUID },
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const GUID_LENGTH: usize = 37;
const BEGIN: &[u8] = b"BEGIN\r\n";

#[derive(Debug, PartialEq, Eq)]
pub enum AuthNextAction {
    Write(&'static [u8]),
    Read(usize),
    Done(GUID),
}

impl AuthFSM {
    pub fn new() -> Self {
        Self::WritingZero
    }

    pub fn next_action(&self) -> AuthNextAction {
        match self {
            Self::WritingZero => AuthNextAction::Write(b"\0"),
            Self::WritingAuthExternal { written } => {
                let rem = &AUTH_EXTERNAL[*written..];
                AuthNextAction::Write(rem)
            }
            Self::ReadingData { read, buf } => {
                let rem = &buf[*read..];
                AuthNextAction::Read(rem.len())
            }
            Self::WritingData { written } => {
                let rem = &DATA[*written..];
                AuthNextAction::Write(rem)
            }
            Self::ReadingGUID { read, buf } => {
                let rem = &buf[*read..];
                AuthNextAction::Read(rem.len())
            }
            Self::WritingBegin { written, .. } => {
                let rem = &BEGIN[*written..];
                AuthNextAction::Write(rem)
            }
            Self::Done { guid } => AuthNextAction::Done(*guid),
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
                        read: 0,
                        buf: [0; DATA.len()],
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
                        read: 0,
                        buf: [0; GUID_LENGTH],
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
                    *self = Self::Done { guid: *guid }
                }
                Ok(())
            }
            Self::Done { .. } => {
                bail!("malformed state, FSM is DONE (in {self:?})");
            }
        }
    }

    pub fn done_reading(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::WritingZero => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::WritingAuthExternal { .. } => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::ReadingData { read, buf } => {
                let start_idx = *read;
                let end_idx = start_idx + data.len();
                let Some(range) = buf.get_mut(start_idx..end_idx) else {
                    bail!("can't get range {start_idx}..{end_idx}");
                };
                range.copy_from_slice(data);
                *read += data.len();
                if *read == DATA.len() {
                    ensure!(buf == DATA);
                    *self = Self::WritingData { written: 0 };
                }
                Ok(())
            }
            Self::WritingData { .. } => {
                bail!("malformed state, you were supposed to WRITE, not READ (in {self:?})");
            }
            Self::ReadingGUID { read, buf } => {
                let start_idx = *read;
                let end_idx = start_idx + data.len();
                let Some(range) = buf.get_mut(start_idx..end_idx) else {
                    bail!("can't get range {start_idx}..{end_idx}");
                };
                range.copy_from_slice(data);
                *read += data.len();
                if *read == GUID_LENGTH {
                    ensure!(&buf[..3] == b"OK ");
                    ensure!(&buf[GUID_LENGTH - 2..] == b"\r\n");
                    let guid = GUID(*buf);
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
    assert_eq!(fsm.next_action(), AuthNextAction::Read(DATA.len()));

    fsm.done_reading(b"DAT").unwrap();
    assert_eq!(fsm.next_action(), AuthNextAction::Read(DATA.len() - 3));

    fsm.done_reading(b"A\r\n").unwrap();
    assert_eq!(fsm.next_action(), AuthNextAction::Write(DATA));

    fsm.done_writing(3).unwrap();
    assert_eq!(fsm.next_action(), AuthNextAction::Write(b"A\r\n"));

    fsm.done_writing(3).unwrap();
    assert_eq!(fsm.next_action(), AuthNextAction::Read(GUID_LENGTH));

    fsm.done_reading(b"OK a97099b37b54cdc2a686559c6922fdeb\r\n")
        .unwrap();
    assert_eq!(fsm.next_action(), AuthNextAction::Write(BEGIN));

    fsm.done_writing(BEGIN.len()).unwrap();
    match fsm.next_action() {
        AuthNextAction::Done(guid) => {
            assert_eq!(guid.as_str().unwrap(), "a97099b37b54cdc2a686559c6922fdeb");
        }
        other => {
            panic!("expected Done, got {other:?}");
        }
    }
}
