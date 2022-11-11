//! # mio_channel
//! Provide a wrapper of the standard channel that can be polled with Mio.
mod channel;

pub use channel::{channel, Sender, Receiver};
