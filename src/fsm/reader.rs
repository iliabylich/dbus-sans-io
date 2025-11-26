use crate::{
    decoders::{DecodingBuffer, HeaderDecoder},
    fsm::ReadBuffer,
};
use anyhow::{Context as _, Result, bail};

#[derive(Debug)]
pub enum ReaderFSM {
    ReadingHeader { buf: ReadBuffer },
    ReadingRest { buf: ReadBuffer },
    Done { buf: Vec<u8> },
}

#[derive(Debug)]
pub enum ReaderNextAction<'a> {
    Read(&'a mut [u8]),
    Message(Vec<u8>),
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self::ReadingHeader {
            buf: ReadBuffer::new(HeaderDecoder::LENGTH + std::mem::size_of::<u32>()),
        }
    }

    pub fn next_action(&mut self) -> ReaderNextAction<'_> {
        match self {
            Self::ReadingHeader { buf } => ReaderNextAction::Read(buf.remaining_part()),
            Self::ReadingRest { buf, .. } => ReaderNextAction::Read(buf.remaining_part()),
            Self::Done { buf } => {
                let buf = std::mem::take(buf);
                *self = Self::new();
                ReaderNextAction::Message(buf)
            }
        }
    }

    pub fn done_reading(&mut self, len: usize) -> Result<()> {
        match self {
            Self::ReadingHeader { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    let (header, header_fields_len) = {
                        let mut buf = DecodingBuffer::new(buf.filled_part());
                        let header = HeaderDecoder::decode(&mut buf)?;
                        let header_fields_len = buf.peek_u32().context("EOF")? as usize;
                        (header, header_fields_len)
                    };

                    let mut new_size = HeaderDecoder::LENGTH + header_fields_len;
                    new_size = new_size.next_multiple_of(8);
                    new_size += header.body_len;
                    buf.resize(new_size);

                    *self = Self::ReadingRest { buf: buf.take() }
                }

                Ok(())
            }
            Self::ReadingRest { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    *self = Self::Done {
                        buf: buf.take().into_vec(),
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
