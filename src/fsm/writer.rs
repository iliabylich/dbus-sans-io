use anyhow::{Context, Result};
use std::collections::VecDeque;

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

    pub(crate) fn next_action(&self) -> WriterNextAction<'_> {
        let Some((pos, buf)) = self.queue.front() else {
            return WriterNextAction::Nothing;
        };

        WriterNextAction::Write(&buf[*pos..])
    }

    pub(crate) fn done_writing(&mut self, len: usize) -> Result<()> {
        let (pos, buf) = self.queue.front_mut().context("malformed state")?;
        *pos += len;
        assert!(*pos <= buf.len());

        if *pos == buf.len() {
            self.queue.pop_front();
        }

        Ok(())
    }
}

pub(crate) enum WriterNextAction<'a> {
    Write(&'a [u8]),
    Nothing,
}
