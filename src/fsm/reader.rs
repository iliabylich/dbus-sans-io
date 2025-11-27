use crate::{
    decoders::{DecodingBuffer, HeaderDecoder, MessageDecoder},
    fsm::{FSMSatisfy, FSMWants, ReadBuffer},
    types::Message,
};
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub struct ReaderFSM {
    state: State,
    buf: ReadBuffer,
}

#[derive(Debug)]
enum State {
    ReadingHeader,
    ReadingFullMessage,
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self {
            state: State::ReadingHeader,
            buf: ReadBuffer::new(HeaderDecoder::LENGTH + std::mem::size_of::<u32>()),
        }
    }

    pub fn wants(&mut self) -> FSMWants<'_> {
        FSMWants::Read(self.buf.remaining_part_mut())
    }

    pub fn satisfy(&mut self, with: FSMSatisfy) -> Result<Option<Message>> {
        let len = with.require_read()?;
        self.buf.add_pos(len);
        if !self.buf.is_full() {
            return Ok(None);
        }

        match self.state {
            State::ReadingHeader => {
                let (header, header_fields_len) = {
                    let mut buf = DecodingBuffer::new(self.buf.filled_part());
                    let header = HeaderDecoder::decode(&mut buf)?;
                    let header_fields_len = buf.peek_u32().context("EOF")? as usize;
                    (header, header_fields_len)
                };

                let mut new_size = HeaderDecoder::LENGTH + header_fields_len;
                new_size = new_size.next_multiple_of(8);
                new_size += header.body_len;
                self.buf.resize(new_size);

                self.state = State::ReadingFullMessage;
                Ok(None)
            }

            State::ReadingFullMessage => {
                let buf = self.buf.take().into_vec();
                let message = MessageDecoder::decode(buf)?;
                *self = Self::new();
                Ok(Some(message))
            }
        }
    }
}
