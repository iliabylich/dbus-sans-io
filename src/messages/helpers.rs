#[macro_export]
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

#[macro_export]
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

#[macro_export]
macro_rules! destination_is {
    ($destination:expr, $expected:expr) => {{
        if $destination != $expected {
            anyhow::bail!(
                "expected destination to be {:?}, got {:?}",
                $expected,
                $destination
            );
        }
    }};
}

#[macro_export]
macro_rules! path_is {
    ($path:expr, $expected:expr) => {{
        if $path != $expected {
            anyhow::bail!("expected path to be {:?}, got {:?}", $expected, $path);
        }
    }};
}

#[macro_export]
macro_rules! member_is {
    ($member:expr, $expected:expr) => {{
        if $member != $expected {
            anyhow::bail!("expected member to be {:?}, got {:?}", $expected, $member);
        }
    }};
}

pub fn as_array<T, const N: usize>(slice: &[T]) -> Option<&[T; N]> {
    slice.try_into().ok()
}

#[macro_export]
macro_rules! body_is {
    ($body:expr, $expected:pat) => {
        let Some($expected) = $crate::messages::as_array($body) else {
            anyhow::bail!("body format mismatch: {:?}", $body);
        };
    };
}

#[macro_export]
macro_rules! value_is {
    ($value:expr, $pat:pat) => {
        let $pat = $value else {
            anyhow::bail!("value format mismatch: {:?}", $value);
        };
    };
}

#[macro_export]
macro_rules! type_is {
    ($type:expr, $pat:pat) => {
        let $pat = $type else {
            anyhow::bail!("type mismatch: {:?}", $type);
        };
    };
}

#[macro_export]
macro_rules! define_sum_message {
    ($name:ident, $($variant:ident),+) => {
        #[derive(Debug)]
        enum $name<'a> {
            $($variant($variant<'a>)),+
        }

        impl<'a> TryFrom<&'a $crate::Message> for $name<'a> {
            type Error = anyhow::Error;

            fn try_from(message: &'a $crate::Message) -> anyhow::Result<Self> {
                $(
                    if let Ok(mapped) = $variant::try_from(message) {
                        return Ok(Self::$variant(mapped));
                    }
                )+

                anyhow::bail!("{message:?} doesn't match any registered type")
            }
        }
    };
}
