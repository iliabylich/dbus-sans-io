use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FSMSatisfy {
    Read { len: usize },
    Write { len: usize },
}

impl FSMSatisfy {
    pub(crate) fn require_read(self) -> Result<usize> {
        match self {
            Self::Read { len } => Ok(len),
            Self::Write { .. } => bail!("expected FSMStatisfy::Read, got Write"),
        }
    }

    pub(crate) fn require_write(self) -> Result<usize> {
        match self {
            Self::Read { .. } => bail!("expected FSMStatisfy::Write, got Read"),
            Self::Write { len } => Ok(len),
        }
    }
}
