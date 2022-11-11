//! # mio_channel
//!
//! Provide a wrapper of the standard channel that can be polled with Mio.
//!
//! ## Example
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     const CHANNEL: mio::Token = mio::Token(0);
//!
//!     #[test]
//!     fn test_channel() -> Result<(), Box<dyn std::error::Error>> {
//!         let mut poll = mio::Poll::new()?;
//!
//!         let mut events = mio::Events::with_capacity(2);
//!
//!         let (tx, mut rx) = mio_channel::channel();
//!
//!         poll.registry().register(&mut rx, CHANNEL, mio::Interest::READABLE)?;
//!
//!         let handler = std::thread::spawn(move || {
//!             std::thread::sleep(std::time::Duration::from_millis(1000));
//!
//!             let _ = tx.send("Hello world!");
//!         });
//!
//!         poll.poll(&mut events, None)?;
//!
//!         assert_eq!(rx.try_recv()?, "Hello world!");
//!
//!         let _ = handler.join();
//!
//!         Ok(())
//!     }
//! }
//! ```

mod channel;

pub use channel::{channel, Sender, Receiver};
