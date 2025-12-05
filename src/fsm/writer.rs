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

    // pub(crate) fn enqueue(&mut self, message: &Message) -> Result<()> {
    //     let buf = MessageEncoder::encode(message)?;
    //     self.enqueue_serialized(buf);
    //     Ok(())
    // }

    pub(crate) fn enqueue(&mut self, buf: Vec<u8>) {
        self.queue.push_back(QueueItem { pos: 0, buf });
    }

    pub(crate) fn wants(&self) -> Option<&[u8]> {
        let QueueItem { pos, buf } = self.queue.front()?;
        Some(&buf[*pos..])
    }

    pub(crate) fn satisfy(&mut self, written: usize) -> Result<()> {
        let QueueItem { pos, buf } = self.queue.front_mut().context("malformed state")?;
        *pos += written;
        assert!(*pos <= buf.len());

        if *pos == buf.len() {
            self.queue.pop_front();
        }

        Ok(())
    }
}
