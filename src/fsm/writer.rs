use crate::fsm::{FSMSatisfy, FSMWants};
use anyhow::{Context, Result};
use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) struct WriterFSM {
    queue: VecDeque<(usize, Vec<u8>)>,
}

impl WriterFSM {
    pub(crate) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, buf: Vec<u8>) {
        self.queue.push_back((0, buf));
    }

    pub(crate) fn wants(&self) -> FSMWants<'_> {
        let Some((pos, buf)) = self.queue.front() else {
            return FSMWants::Nothing;
        };

        FSMWants::Write(&buf[*pos..])
    }

    pub(crate) fn satisfy(&mut self, with: FSMSatisfy) -> Result<()> {
        let len = with.require_write()?;

        let (pos, buf) = self.queue.front_mut().context("malformed state")?;
        *pos += len;
        assert!(*pos <= buf.len());

        if *pos == buf.len() {
            self.queue.pop_front();
        }

        Ok(())
    }
}
