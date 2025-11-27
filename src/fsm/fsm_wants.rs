#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FSMWants<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
    Nothing,
}
