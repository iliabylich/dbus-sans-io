use crate::{
    decoders::{DecodingBuffer, HeaderDecoder},
    fsm::ReadBuffer,
    types::Header,
};
use anyhow::{Result, bail};

#[derive(Debug)]
pub enum ReaderFSM {
    ReadingHeadar { buf: ReadBuffer },
    ReadingBody { buf: ReadBuffer },
    Done { buf: Vec<u8> },
}

pub enum ReaderNextAction<'a> {
    Read(&'a mut [u8]),
    Message(Vec<u8>),
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self::ReadingHeadar {
            buf: ReadBuffer::new(Header::LENGTH),
        }
    }

    pub fn next_action(&mut self) -> ReaderNextAction<'_> {
        match self {
            Self::ReadingHeadar { buf } => ReaderNextAction::Read(buf.remaining_part()),
            Self::ReadingBody { buf } => ReaderNextAction::Read(buf.remaining_part()),
            Self::Done { buf } => {
                let buf = std::mem::take(buf);
                *self = Self::new();
                ReaderNextAction::Message(buf)
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
                    *self = Self::Done {
                        buf: buf.take().unwrap(),
                    }
                }

                Ok(())
            }
            Self::Done { .. } => {
                bail!("malformed state, you were supposed to take message, not READ (in {self:?})")
            }
        }
    }
}
