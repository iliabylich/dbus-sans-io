use anyhow::{Result, bail};

use crate::{
    fsm::{AuthFSM, AuthNextAction, ReaderFSM, ReaderNextAction, WriterFSM},
    types::GUID,
};

pub(crate) enum FullFSM {
    Auth(AuthFSM),
    ReadWrite {
        reader: ReaderFSM,
        writer: WriterFSM,
    },
}

pub(crate) enum ReadAction<'a> {
    Read(&'a mut [u8]),
    AuthDone(Vec<u8>),
    Message(Vec<u8>),
    Nothing,
}

pub(crate) enum WriteAction<'a> {
    Write(&'a [u8]),
    Nothing,
}

impl FullFSM {
    pub(crate) fn new() -> Self {
        Self::Auth(AuthFSM::new())
    }

    pub(crate) fn enqueue(&mut self, buf: Vec<u8>) -> Result<()> {
        let Self::ReadWrite { writer, .. } = self else {
            bail!("auth must be completed before enqueuing a message");
        };
        writer.enqueue(buf);
        Ok(())
    }

    pub(crate) fn next_action_if_readable(&mut self) -> ReadAction<'_> {
        let this: *mut Self = self;

        match self {
            Self::Auth(auth) => match auth.next_action() {
                AuthNextAction::Write(_) => ReadAction::Nothing,
                AuthNextAction::Read(buf) => ReadAction::Read(buf),
                AuthNextAction::Done(guid) => {
                    // SAFETY: FullFSM doesn't expose any references
                    // and previous value of `self` is not used anywhere
                    unsafe {
                        this.write(Self::ReadWrite {
                            reader: ReaderFSM::new(),
                            writer: WriterFSM::new(),
                        });
                    }
                    ReadAction::AuthDone(guid)
                }
            },
            Self::ReadWrite { reader, .. } => match reader.next_action() {
                ReaderNextAction::Read(buf) => ReadAction::Read(buf),
                ReaderNextAction::Message(buf) => ReadAction::Message(buf),
            },
        }
    }
}
