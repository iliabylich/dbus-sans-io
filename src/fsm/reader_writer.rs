use crate::{
    fsm::{FSMSatisfy, FSMWants, ReaderFSM, WriterFSM},
    types::Message,
};
use anyhow::Result;

pub(crate) struct ReaderWriterFSM {
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl ReaderWriterFSM {
    pub(crate) fn new() -> Self {
        Self {
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &Message) -> Result<()> {
        self.writer.enqueue(message)
    }

    pub(crate) fn wants(&mut self) -> FSMWants<'_> {
        let writer_wants = self.writer.wants();
        if matches!(writer_wants, FSMWants::Write(_)) {
            return writer_wants;
        }

        self.reader.wants()
    }

    pub(crate) fn satisfy(&mut self, with: FSMSatisfy) -> Result<Option<Message>> {
        match with {
            FSMSatisfy::Read { .. } => match self.reader.satisfy(with)? {
                Some(message) => Ok(Some(message)),
                None => Ok(None),
            },
            FSMSatisfy::Write { .. } => {
                self.writer.satisfy(with)?;
                Ok(None)
            }
        }
    }
}
