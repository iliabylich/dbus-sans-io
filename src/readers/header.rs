use crate::{DynamicSizeReader, FixedSizeReader, HeaderField, IoOperation, IoReader, MessageType};
use anyhow::Result;
use std::io::Read;

pub(crate) enum HeaderReader {
    ReadingPreHeader(FixedSizeReader<16>),
    ReadingHeaderFields {
        pre_header: PreHeader,
        reader: DynamicSizeReader,
    },
    ReadingPaddingLeftover {
        header: Header,
        reader: DynamicSizeReader,
    },
}

impl HeaderReader {
    pub(crate) fn new() -> Self {
        Self::ReadingPreHeader(FixedSizeReader::new())
    }
}

impl IoReader<Header> for HeaderReader {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<Header>> {
        use HeaderReader::*;
        use IoOperation::*;

        loop {
            match self {
                ReadingPreHeader(metadata) => match metadata.continue_reading(r)? {
                    Finished(bytes) => {
                        let pre_header = PreHeader::try_from(bytes)?;
                        println!("{pre_header:?}");
                        *self = ReadingHeaderFields {
                            reader: DynamicSizeReader::new(pre_header.header_fields_len),
                            pre_header,
                        }
                    }
                    WouldBlock => return Ok(WouldBlock),
                },

                ReadingHeaderFields { pre_header, reader } => match reader.continue_reading(r)? {
                    Finished(bytes) => {
                        let pre_header = std::mem::take(pre_header);
                        let header_fields = HeaderFields::try_from(bytes)?;

                        let read_so_far = 16 + pre_header.header_fields_len;
                        let padding_len = read_so_far.next_multiple_of(8) - read_so_far;
                        println!("padding len = {padding_len}");

                        let header = Header::new(pre_header, header_fields);

                        *self = ReadingPaddingLeftover {
                            header,
                            reader: DynamicSizeReader::new(padding_len),
                        };
                    }
                    WouldBlock => return Ok(WouldBlock),
                },

                ReadingPaddingLeftover { header, reader } => match reader.continue_reading(r)? {
                    Finished(_padding) => {
                        let header = std::mem::take(header);
                        return Ok(Finished(header));
                    }
                    WouldBlock => return Ok(WouldBlock),
                },
            }
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct PreHeader {
    message_type: MessageType,
    flags: u8,
    body_len: usize,
    serial: u32,
    header_fields_len: usize,
}

impl TryFrom<[u8; 16]> for PreHeader {
    type Error = anyhow::Error;

    fn try_from(data: [u8; 16]) -> Result<Self> {
        let message_type = MessageType::try_from(data[1])?;
        let flags = data[2];
        let body_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let serial = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let header_fields_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        Ok(Self {
            message_type,
            flags,
            body_len: body_len as usize,
            serial,
            header_fields_len: header_fields_len as usize,
        })
    }
}

#[derive(Default, Debug)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: u8,
    pub(crate) serial: u32,
    pub(crate) body_len: usize,
    pub(crate) member: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) path: Option<String>,
}

impl Header {
    fn new(
        PreHeader {
            message_type,
            flags,
            body_len,
            serial,
            header_fields_len: _,
        }: PreHeader,
        HeaderFields {
            member,
            interface,
            path,
        }: HeaderFields,
    ) -> Self {
        Self {
            message_type,
            flags,
            serial,
            member,
            interface,
            path,
            body_len,
        }
    }
}

#[derive(Default, Debug)]
struct HeaderFields {
    member: Option<String>,
    interface: Option<String>,
    path: Option<String>,
}

impl TryFrom<Vec<u8>> for HeaderFields {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        let mut member = None;
        let mut interface = None;
        let mut path = None;
        let mut pos = 0;

        while pos < bytes.len() {
            // Align to 8-byte boundary from message start
            let absolute_pos = 16 + pos;
            let padding = (8 - (absolute_pos % 8)) % 8;
            pos += padding;

            if pos >= bytes.len() {
                break;
            }

            let field_code = bytes[pos];
            pos += 1;
            let _sig_len = bytes[pos];
            pos += 1;
            let signature = bytes[pos];
            pos += 1;
            pos += 1; // skip signature null terminator

            match signature {
                b's' | b'o' => {
                    let str_len = u32::from_le_bytes([
                        bytes[pos],
                        bytes[pos + 1],
                        bytes[pos + 2],
                        bytes[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let value = String::from_utf8_lossy(&bytes[pos..pos + str_len]).into_owned();
                    pos += str_len + 1; // +1 for null terminator

                    if let Ok(field) = HeaderField::try_from(field_code) {
                        match field {
                            HeaderField::Path => path = Some(value),
                            HeaderField::Interface => interface = Some(value),
                            HeaderField::Member => member = Some(value),
                            _ => {}
                        }
                    }
                }
                _ => break, // Skip unknown signatures for now
            }
        }

        Ok(Self {
            member,
            interface,
            path,
        })
    }
}
