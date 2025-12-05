use crate::{Message, fsm::ReaderFSM};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct BufferedReaderFSM {
    reader: ReaderFSM,
    last: *mut u8,
}
impl BufferedReaderFSM {
    pub(crate) fn new() -> Self {
        Self {
            reader: ReaderFSM::new(),
            last: std::ptr::null_mut(),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<&mut [u8]> {
        let new = self.reader.wants();
        if new.as_ptr() != self.last {
            self.last = new.as_mut_ptr();
            Some(new)
        } else {
            None
        }
    }

    pub(crate) fn satisfy(&mut self, read: usize) -> Result<Option<Message>> {
        self.reader.satisfy(read)
    }
}
