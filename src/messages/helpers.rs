macro_rules! message_is {
    ($message:expr, $pat:pat) => {
        let $pat = $message else {
            anyhow::bail!(
                "expected Message::{}, got {:?}",
                stringify!($expected),
                $message
            );
        };
    };
}
pub(crate) use message_is;

macro_rules! interface_is {
    ($interface:expr, $expected:expr) => {{
        if $interface != $expected {
            anyhow::bail!(
                "expected interface to be {:?}, got {:?}",
                $expected,
                $interface
            );
        }
    }};
}
pub(crate) use interface_is;

macro_rules! path_is {
    ($path:expr, $expected:expr) => {{
        if $path != $expected {
            anyhow::bail!("expected path to be {:?}, got {:?}", $expected, $path);
        }
    }};
}
pub(crate) use path_is;

pub(crate) fn as_array<T, const N: usize>(slice: &[T]) -> Option<&[T; N]> {
    if slice.len() == N {
        let ptr = slice.as_ptr().cast();

        // SAFETY: The underlying array of a slice can be reinterpreted as an actual array `[T; N]` if `N` is not greater than the slice's length.
        let me = unsafe { &*ptr };
        Some(me)
    } else {
        None
    }
}
macro_rules! body_is {
    ($body:expr, $expected:pat) => {
        let Some($expected) = $crate::messages::helpers::as_array($body) else {
            anyhow::bail!("body format mismatch: {:?}", $body);
        };
    };
}
pub(crate) use body_is;

macro_rules! value_is {
    ($value:expr, $pat:pat) => {
        let $pat = $value else {
            anyhow::bail!("value format mismatch: {:?}", $value);
        };
    };
}
pub(crate) use value_is;

macro_rules! type_is {
    ($type:expr, $pat:pat) => {
        let $pat = $type else {
            anyhow::bail!("type mismatch: {:?}", $type);
        };
    };
}
pub(crate) use type_is;
