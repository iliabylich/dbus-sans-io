use anyhow::Result;
use std::io::{Read, Write};

#[must_use]
#[derive(Debug)]
pub enum IoOperation<T> {
    Finished(T),
    WouldBlock,
}

impl<T> IoOperation<T> {
    pub fn map<F, U>(self, f: F) -> IoOperation<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Finished(value) => IoOperation::Finished(f(value)),
            Self::WouldBlock => IoOperation::WouldBlock,
        }
    }
}

pub trait IoReader<T> {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<T>>;
}

pub trait IoWriter<T> {
    fn continue_writing(&mut self, w: &mut impl Write) -> Result<IoOperation<T>>;
}

pub trait IoRoundtrip<T> {
    fn continue_roundtrip(&mut self, rw: &mut (impl Read + Write)) -> Result<IoOperation<T>>;
}
