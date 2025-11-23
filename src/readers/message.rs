use crate::{
    DynamicSizeReader, FixedSizeReader, IoOperation, IoReader, Message, MessageParser,
    parsers::{Header, HeaderFields, HeaderFieldsParser, HeaderParser},
};
use anyhow::Result;
use std::io::Read;

pub(crate) enum MessageReader {
    ReadingHeader {
        reader: FixedSizeReader<16>,
    },
    ReadingHeaderFields {
        reader: DynamicSizeReader,
        header: Header,
    },
    ReadingPaddingLeftover {
        reader: DynamicSizeReader,
        header: Header,
        header_fields: HeaderFields,
    },
    ReadingBody {
        reader: DynamicSizeReader,
        header: Header,
        header_fields: HeaderFields,
    },
}

impl MessageReader {
    pub(crate) fn new() -> Self {
        Self::ReadingHeader {
            reader: FixedSizeReader::new(),
        }
    }
}

impl IoReader<Message> for MessageReader {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<Message>> {
        use IoOperation::*;
        use MessageReader::*;

        loop {
            match self {
                ReadingHeader { reader } => match reader.continue_reading(r)? {
                    Finished(bytes) => {
                        let header = HeaderParser::parse(bytes)?;
                        *self = ReadingHeaderFields {
                            reader: DynamicSizeReader::new(header.header_fields_len),
                            header,
                        }
                    }
                    WouldBlock => return Ok(WouldBlock),
                },
                ReadingHeaderFields { header, reader } => match reader.continue_reading(r)? {
                    Finished(bytes) => {
                        let header = std::mem::take(header);
                        let header_fields = HeaderFieldsParser::parse(bytes)?;

                        let read_so_far = 16 + header.header_fields_len;
                        let padding_len = read_so_far.next_multiple_of(8) - read_so_far;
                        println!("padding len = {padding_len}");

                        *self = ReadingPaddingLeftover {
                            header,
                            header_fields,
                            reader: DynamicSizeReader::new(padding_len),
                        }
                    }
                    WouldBlock => return Ok(WouldBlock),
                },
                ReadingPaddingLeftover {
                    header,
                    header_fields,
                    reader,
                } => match reader.continue_reading(r)? {
                    Finished(_padding) => {
                        let header = std::mem::take(header);
                        let header_fields = std::mem::take(header_fields);
                        *self = ReadingBody {
                            reader: DynamicSizeReader::new(header.body_len),
                            header,
                            header_fields,
                        };
                    }
                    WouldBlock => return Ok(WouldBlock),
                },
                ReadingBody {
                    header,
                    header_fields,
                    reader,
                } => match reader.continue_reading(r)? {
                    Finished(bytes) => {
                        let header = std::mem::take(header);
                        let header_fields = std::mem::take(header_fields);
                        return Ok(Finished(Message::new(header, header_fields, bytes)));
                    }
                    WouldBlock => return Ok(WouldBlock),
                },
            }
        }
    }
}
