use anyhow::{Context, Result};
use std::os::unix::net::UnixStream;

mod decoders;
mod encoders;
mod fsm;
mod serial;
mod types;

#[cfg(feature = "blocking")]
mod blocking_connection;
#[cfg(feature = "blocking")]
pub use blocking_connection::BlockingConnection;

#[cfg(feature = "poll")]
mod poll_connection;
#[cfg(feature = "poll")]
pub use poll_connection::PollConnection;

#[cfg(feature = "io-uring")]
mod io_uring_connection;
#[cfg(feature = "io-uring")]
pub use io_uring_connection::IoUringConnection;

pub use types::{CompleteType, Message, Value};
pub mod messages;

#[allow(dead_code)]
pub(crate) fn session_connection() -> Result<UnixStream> {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
    let (_, path) = address
        .split_once("=")
        .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
    let stream = UnixStream::connect(path)?;
    Ok(stream)
}

#[test]
fn test_encode_decode_hello() {
    use crate::{decoders::MessageDecoder, encoders::MessageEncoder, messages::Hello};
    let message = Hello.into();
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(decoded, Hello.into());
}

#[test]
fn test_encode_decode_show_notification() {
    use crate::{decoders::MessageDecoder, encoders::MessageEncoder, messages::ShowNotification};
    let message = ShowNotification::new("Header", "Body").into();
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(decoded, ShowNotification::new("Header", "Body").into());
}
