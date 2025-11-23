use crate::HeaderField;
use anyhow::Result;

#[derive(Default, Debug)]
pub(crate) struct HeaderFields {
    pub(crate) member: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) path: Option<String>,
}

pub(crate) struct HeaderFieldsParser;

impl HeaderFieldsParser {
    pub(crate) fn parse(bytes: Vec<u8>) -> Result<HeaderFields> {
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

        Ok(HeaderFields {
            member,
            interface,
            path,
        })
    }
}
