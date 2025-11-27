use crate::{
    decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    fsm::{FSMSatisfy, FSMWants, ReadBuffer},
    types::Message,
};
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub enum ReaderFSM {
    ReadingHeader { buf: ReadBuffer },
    ReadingRest { buf: ReadBuffer },
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self::ReadingHeader {
            buf: ReadBuffer::new(HeaderDecoder::LENGTH + std::mem::size_of::<u32>()),
        }
    }

    pub fn wants(&mut self) -> FSMWants<'_> {
        match self {
            Self::ReadingHeader { buf } => FSMWants::Read(buf.remaining_part_mut()),
            Self::ReadingRest { buf, .. } => FSMWants::Read(buf.remaining_part_mut()),
        }
    }

    pub fn satisfy(&mut self, with: FSMSatisfy) -> Result<Option<Message>> {
        let len = with.require_read()?;

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

                Ok(None)
            }

            Self::ReadingRest { buf } => {
                buf.add_pos(len);
                if buf.is_full() {
                    let buf = buf.take().into_vec();
                    let message = MessageDecoder::decode(buf)?;
                    *self = Self::new();
                    Ok(Some(message))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
