#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub enum HeaderField {
    #[default]
    Invalid = 0,
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
    UnixFds = 9,
}

impl From<u8> for HeaderField {
    fn from(byte: u8) -> Self {
        match byte {
            1 => HeaderField::Path,
            2 => HeaderField::Interface,
            3 => HeaderField::Member,
            4 => HeaderField::ErrorName,
            5 => HeaderField::ReplySerial,
            6 => HeaderField::Destination,
            7 => HeaderField::Sender,
            8 => HeaderField::Signature,
            _ => HeaderField::Invalid,
        }
    }
}

impl From<HeaderField> for u8 {
    fn from(header_field: HeaderField) -> Self {
        match header_field {
            HeaderField::Invalid => 0,
            HeaderField::Path => 1,
            HeaderField::Interface => 2,
            HeaderField::Member => 3,
            HeaderField::ErrorName => 4,
            HeaderField::ReplySerial => 5,
            HeaderField::Destination => 6,
            HeaderField::Sender => 7,
            HeaderField::Signature => 8,
            HeaderField::UnixFds => 9,
        }
    }
}
