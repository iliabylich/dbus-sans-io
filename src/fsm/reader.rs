use crate::{Message, fsm::ReadBuffer, parsers::Header};
use anyhow::{Result, bail};

const HEADER_LEN: usize = 16;

#[derive(Debug)]
pub enum ReaderFSM {
    ReadingHeadar { buf: ReadBuffer },
    ReadingBody { buf: ReadBuffer },
    Done { message: Message },
}

pub enum ReaderNextAction<'a> {
    Read(&'a mut [u8]),
    Message(Message),
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self::new_reading_header()
    }

    fn new_reading_header() -> Self {
        Self::ReadingHeadar {
            buf: ReadBuffer::new(HEADER_LEN),
        }
    }

    pub fn next_action(&mut self) -> ReaderNextAction<'_> {
        match self {
            Self::ReadingHeadar { buf } => ReaderNextAction::Read(buf.remainder()),
            Self::ReadingBody { buf } => ReaderNextAction::Read(buf.remainder()),
            Self::Done { message } => {
                let message = std::mem::take(message);
                *self = Self::new();
                ReaderNextAction::Message(message)
            }
        }
    }

    pub fn done_reading(&mut self, len: usize) -> Result<()> {
        match self {
            Self::ReadingHeadar { buf } => {
                buf.written(len);
                if buf.is_full() {
                    let header = Header::new(buf.as_bytes())?;
                    let mut new_size = header.header_fields_len();
                    new_size = new_size.next_multiple_of(8);
                    new_size += header.body_len();
                    buf.grow(new_size);
                    *self = Self::ReadingBody { buf: buf.take() }
                }

                Ok(())
            }
            Self::ReadingBody { buf } => {
                buf.written(len);
                if buf.is_full() {
                    let message = Message::split(buf.take().unwrap())?;
                    *self = Self::Done { message }
                }

                Ok(())
            }
            Self::Done { .. } => {
                bail!("malformed state, you were supposed to take message, not READ (in {self:?})")
            }
        }
    }
}
