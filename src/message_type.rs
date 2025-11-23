#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Invalid = 0,
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::MethodCall,
            2 => Self::MethodReturn,
            3 => Self::Error,
            4 => Self::Signal,
            _ => Self::Invalid,
        }
    }
}
