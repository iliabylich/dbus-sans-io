use crate::{
    encoders::MessageEncoder,
    fsm::{FSMSatisfy, FSMWants},
    types::Message,
};
use anyhow::{Context, Result};
use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) struct WriterFSM {
    queue: VecDeque<QueueItem>,
}

#[derive(Debug)]
struct QueueItem {
    pos: usize,
    buf: Vec<u8>,
}

impl WriterFSM {
    pub(crate) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &Message) -> Result<()> {
        let buf = MessageEncoder::encode(message)?;
        self.queue.push_back(QueueItem { pos: 0, buf });
        Ok(())
    }

    pub(crate) fn wants(&self) -> FSMWants<'_> {
        match self.queue.front() {
            Some(QueueItem { pos, buf }) => FSMWants::Write(&buf[*pos..]),
            None => FSMWants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, with: FSMSatisfy) -> Result<()> {
        let len = with.require_write()?;

        let QueueItem { pos, buf } = self.queue.front_mut().context("malformed state")?;
        *pos += len;
        assert!(*pos <= buf.len());

        if *pos == buf.len() {
            self.queue.pop_front();
        }

        Ok(())
    }
}
