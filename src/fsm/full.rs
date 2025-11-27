use crate::{
    fsm::{AuthFSM, FSMSatisfy, FSMWants, ReaderFSM, WriterFSM},
    types::{GUID, Message},
};
use anyhow::{Result, bail};

#[derive(Debug)]
pub(crate) enum FullFSM {
    Auth(AuthFSM),
    ReadWrite {
        reader: ReaderFSM,
        writer: WriterFSM,
    },
}

pub(crate) enum Output {
    GUID(GUID),
    Message(Message),
    NothingYet,
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

    pub(crate) fn wants(&mut self) -> FSMWants<'_> {
        match self {
            FullFSM::Auth(auth) => auth.wants(),
            FullFSM::ReadWrite { reader, writer } => {
                let writer_wants = writer.wants();
                if matches!(writer_wants, FSMWants::Write(_)) {
                    return writer_wants;
                }

                reader.wants()
            }
        }
    }

    pub(crate) fn satisfy(&mut self, with: FSMSatisfy) -> Result<Output> {
        match self {
            FullFSM::Auth(auth) => match auth.satisfy(with)? {
                Some(guid) => {
                    *self = Self::ReadWrite {
                        reader: ReaderFSM::new(),
                        writer: WriterFSM::new(),
                    };
                    Ok(Output::GUID(guid))
                }
                None => Ok(Output::NothingYet),
            },
            FullFSM::ReadWrite { reader, writer } => match with {
                FSMSatisfy::Read { .. } => match reader.satisfy(with)? {
                    Some(message) => Ok(Output::Message(message)),
                    None => Ok(Output::NothingYet),
                },
                FSMSatisfy::Write { .. } => {
                    writer.satisfy(with)?;
                    Ok(Output::NothingYet)
                }
            },
        }
    }

    // pub(crate) fn next_action(&mut self, ready: Action) -> ReadAction<'_> {
    //     match (self, ready) {
    //         (FullFSM::Auth(auth), Action::Read) => todo!(),
    //         (FullFSM::Auth(auth), Action::Write) => todo!(),
    //         (FullFSM::ReadWrite { reader, writer }, Action::Read) => todo!(),
    //         (FullFSM::ReadWrite { reader, writer }, Action::Write) => todo!(),
    //     }

    //     match self {
    //         Self::Auth(auth) => match auth.next_action() {
    //             AuthNextAction::Write(_) => ReadAction::Nothing,
    //             AuthNextAction::Read(buf) => ReadAction::Read(buf),
    //             // AuthNextAction::Done(guid) => {
    //             //     // SAFETY: FullFSM doesn't expose any references
    //             //     // and previous value of `self` is not used anywhere
    //             //     unsafe {
    //             //         this.write(Self::ReadWrite {
    //             //             reader: ReaderFSM::new(),
    //             //             writer: WriterFSM::new(),
    //             //         });
    //             //     }
    //             //     ReadAction::AuthDone(guid)
    //             // }
    //         },
    //         Self::ReadWrite { reader, .. } => match reader.next_action() {
    //             ReaderNextAction::Read(buf) => ReadAction::Read(buf),
    //             ReaderNextAction::Message(buf) => ReadAction::Message(buf),
    //         },
    //     }
    // }

    // pub(crate) fn next_action_if_writable(&mut self) -> WriteAction<'_> {
    //     match self {
    //         Self::Auth(auth) => match auth.next_action() {
    //             AuthNextAction::Write(buf) => WriteAction::Write(buf),
    //             AuthNextAction::Read(buf) => WriteAction::Nothing,
    //         },
    //         Self::ReadWrite { reader, writer } => todo!(),
    //     }
    //     // match self {
    //     //     Self::Auth(auth) => match auth.next_action() {
    //     //         AuthNextAction::Write(buf) => WriteAction::Write(buf),
    //     //         AuthNextAction::Read(_) => WriteAction::Nothing,
    //     //         AuthNextAction::Done(guid) => {
    //     //             todo!()
    //     //         }
    //     //     },
    //     //     Self::ReadWrite { writer, .. } => match writer.next_action() {
    //     //         WriterNextAction::Write(items) => todo!(),
    //     //         WriterNextAction::Nothing => todo!(),
    //     //     },
    //     // }
    // }

    // pub(crate) fn done_reading(&mut self, len: usize) -> Result<ReadResult> {
    //     match self {
    //         Self::Auth(auth) => {
    //             auth.done_reading(len)?;
    //             Ok(ReadResult::Nothing)
    //         }
    //         Self::ReadWrite { reader, .. } => match reader.done_reading(len)? {
    //             Some(message) => Ok(ReadResult::Message(message)),
    //             None => Ok(ReadResult::Nothing),
    //         },
    //     }
    // }

    // pub(crate) fn done_writing(&mut self, len: usize) -> Result<WriteResult> {
    //     match self {
    //         FullFSM::Auth(auth) => match auth.done_writing(len)? {
    //             Some(guid) => {
    //                 *self = Self::ReadWrite {
    //                     reader: ReaderFSM::new(),
    //                     writer: WriterFSM::new(),
    //                 };
    //                 Ok(WriteResult::GUID(guid))
    //             }
    //             None => Ok(WriteResult::Nothing),
    //         },
    //         FullFSM::ReadWrite { writer, .. } => {
    //             writer.done_writing(len)?;
    //             Ok(WriteResult::Nothing)
    //         }
    //     }
    // }
}
