use anyhow::{Result, bail, ensure};

use crate::{
    Message,
    parsers::{Header, HeaderFields, HeaderFieldsParser, HeaderParser},
};

const HEADER_LEN: usize = 16;

#[derive(Debug)]
pub enum ReaderFSM {
    ReadingHeadar {
        buf: [u8; HEADER_LEN],
        read: usize,
    },
    ReadingHeaderFields {
        buf: Vec<u8>,
        read: usize,
        header: Header,
    },
    ReadingPaddingLeftover {
        rem: usize,
        header: Header,
        header_fields: HeaderFields,
    },
    ReadingBody {
        buf: Vec<u8>,
        read: usize,
        header: Header,
        header_fields: HeaderFields,
    },
    Done {
        message: Message,
    },
}

pub enum ReaderNextAction {
    Read(usize),
    Message(Message),
}

impl ReaderFSM {
    pub fn new() -> Self {
        Self::new_reading_header()
    }

    fn new_reading_header() -> Self {
        Self::ReadingHeadar {
            buf: [0; HEADER_LEN],
            read: 0,
        }
    }

    fn new_reading_header_fields(header: Header) -> Result<Self, Header> {
        if header.has_header_fields() {
            Ok(Self::ReadingHeaderFields {
                buf: vec![0; header.header_fields_len],
                read: 0,
                header,
            })
        } else {
            Err(header)
        }
    }

    fn new_reading_padding(
        header: Header,
        header_fields: HeaderFields,
    ) -> Result<Self, (Header, HeaderFields)> {
        if header.has_padding() {
            Ok(Self::ReadingPaddingLeftover {
                rem: header.padding_len(),
                header,
                header_fields,
            })
        } else {
            Err((header, header_fields))
        }
    }

    fn new_reading_body(
        header: Header,
        header_fields: HeaderFields,
    ) -> Result<Self, (Header, HeaderFields)> {
        if header.has_body() {
            Ok(Self::ReadingBody {
                buf: vec![0; header.body_len],
                read: 0,
                header,
                header_fields,
            })
        } else {
            Err((header, header_fields))
        }
    }

    fn new_done(header: Header, header_fields: HeaderFields, body: Vec<u8>) -> Self {
        Self::Done {
            message: Message::new(header, header_fields, body),
        }
    }

    pub fn next_action(&mut self) -> ReaderNextAction {
        match self {
            Self::ReadingHeadar { buf, read } => {
                let rem = &buf[*read..];
                ReaderNextAction::Read(rem.len())
            }
            Self::ReadingHeaderFields { buf, read, .. } => {
                let rem = &buf[*read..];
                ReaderNextAction::Read(rem.len())
            }
            Self::ReadingPaddingLeftover { rem, .. } => ReaderNextAction::Read(*rem),
            Self::ReadingBody { buf, read, .. } => {
                let rem = &buf[*read..];
                ReaderNextAction::Read(rem.len())
            }
            Self::Done { message } => {
                let message = std::mem::take(message);
                *self = Self::new();
                ReaderNextAction::Message(message)
            }
        }
    }

    pub fn done_reading(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::ReadingHeadar { buf, read } => {
                let start_idx = *read;
                let end_idx = start_idx + data.len();
                let Some(range) = buf.get_mut(start_idx..end_idx) else {
                    bail!("can't get range {start_idx}..{end_idx}");
                };
                range.copy_from_slice(data);
                *read += data.len();
                if *read == HEADER_LEN {
                    let header = HeaderParser::parse(*buf)?;

                    *self = Result::<Self, ()>::Err(())
                        .or_else(|_| Self::new_reading_header_fields(header))
                        .or_else(|h| Self::new_reading_padding(h, HeaderFields::default()))
                        .or_else(|(h, hf)| Self::new_reading_body(h, hf))
                        .unwrap_or_else(|(h, hf)| Self::new_done(h, hf, vec![]));
                }
                Ok(())
            }
            Self::ReadingHeaderFields { buf, read, header } => {
                let start_idx = *read;
                let end_idx = start_idx + data.len();
                let Some(range) = buf.get_mut(start_idx..end_idx) else {
                    bail!("can't get range {start_idx}..{end_idx}");
                };
                range.copy_from_slice(data);
                *read += data.len();
                if *read == buf.len() {
                    let header = std::mem::take(header);
                    let buf = std::mem::take(buf);
                    let header_fields = HeaderFieldsParser::parse(buf)?;

                    *self = Result::<Self, ()>::Err(())
                        .or_else(|_| Self::new_reading_padding(header, header_fields))
                        .or_else(|(h, hf)| Self::new_reading_body(h, hf))
                        .unwrap_or_else(|(h, hf)| Self::new_done(h, hf, vec![]));
                }
                Ok(())
            }
            Self::ReadingPaddingLeftover {
                rem,
                header,
                header_fields,
            } => {
                ensure!(data.len() <= *rem);
                *rem -= data.len();
                if *rem == 0 {
                    let header = std::mem::take(header);
                    let header_fields = std::mem::take(header_fields);

                    *self = Result::<Self, ()>::Err(())
                        .or_else(|_| Self::new_reading_body(header, header_fields))
                        .unwrap_or_else(|(h, hf)| Self::new_done(h, hf, vec![]));
                }
                Ok(())
            }
            Self::ReadingBody {
                buf,
                read,
                header,
                header_fields,
            } => {
                let start_idx = *read;
                let end_idx = start_idx + data.len();
                let Some(range) = buf.get_mut(start_idx..end_idx) else {
                    bail!("can't get range {start_idx}..{end_idx}");
                };
                range.copy_from_slice(data);
                *read += data.len();
                if *read == buf.len() {
                    let header = std::mem::take(header);
                    let header_fields = std::mem::take(header_fields);
                    let body = std::mem::take(buf);

                    *self = Self::new_done(header, header_fields, body);
                }
                Ok(())
            }
            Self::Done { message } => {
                bail!("malformed state, you were supposed to take message, not READ (in {self:?})")
            }
        }
    }
}
