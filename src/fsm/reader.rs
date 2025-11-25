use crate::{
    Message,
    decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    fsm::ReadBuffer,
};
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
        Self::ReadingHeadar {
            buf: ReadBuffer::new(HEADER_LEN),
        }
    }

    pub fn next_action(&mut self) -> ReaderNextAction<'_> {
        match self {
            Self::ReadingHeadar { buf } => ReaderNextAction::Read(buf.remaining_part()),
            Self::ReadingBody { buf } => ReaderNextAction::Read(buf.remaining_part()),
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
                buf.add_pos(len);
                if buf.is_full() {
                    let header = HeaderDecoder::decode(DecodingBuffer::new(buf.filled_part()))?;
                    buf.resize(header.full_message_size());
                    *self = Self::ReadingBody { buf: buf.take() }
                }

                Ok(())
            }
            Self::ReadingBody { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    let message = MessageDecoder::decode(buf.take().unwrap())?;
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
