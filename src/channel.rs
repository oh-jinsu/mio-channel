use std::{sync::{mpsc, Arc, Mutex}, io};

use mio::{Waker, event, Token};

/// Create a pair of the [`Sender`] and the [`Receiver`].
/// 
/// The [`Receiver`] implements the [`event::Source`] so that it can be registered
/// with the [`mio::poll::Poll`], while the [`Sender`] doesn't.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel();

    let waker = Arc::new(Mutex::new(None));

    (Sender { waker: waker.clone(), tx }, Receiver { waker, rx })
}

/// Create a pair of the [`SyncSender`] and the [`Receiver`].
///
/// The [`Receiver`] implements the [`event::Source`] so that it can be registered
/// with the [`mio::poll::Poll`], while the [`Sender`] doesn't.
pub fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::sync_channel(bound);

    let waker = Arc::new(Mutex::new(None));

    (SyncSender { waker: waker.clone(), tx }, Receiver { waker, rx })
}

/// A wrapper of the [`mpsc::Receiver`].
/// 
/// It implements the [`event::Source`] so that it can be registered with the [`mio::poll::Poll`].
/// It ignores the [`mio::Interest`] and always cause readable events.
pub struct Receiver<T> {
    waker: Arc<Mutex<Option<Waker>>>,
    rx: mpsc::Receiver<T>
}

impl<T> Receiver<T> {
    /// Try to receive a value. It works just like [`mpsc::Receiver::try_recv`].
    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

impl<T> event::Source for Receiver<T> {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: Token,
        _: mio::Interest,
    ) -> io::Result<()> {
        let mut waker = self.waker.lock().unwrap();

        if waker.is_none() {
            *waker = Some(Waker::new(registry, token)?);
        }

        Ok(())
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: Token,
        _: mio::Interest,
    ) -> io::Result<()> {
        let mut waker = self.waker.lock().unwrap();

        *waker = Some(Waker::new(registry, token)?);
     
        Ok(())
    }

    fn deregister(&mut self, _: &mio::Registry) -> io::Result<()> {
        let mut waker = self.waker.lock().unwrap();

        *waker = None;

        Ok(())
    }
}

/// A wrapper of the [`mpsc::Sender`].
pub struct Sender<T> {
    waker: Arc<Mutex<Option<Waker>>>,
    tx: mpsc::Sender<T>
}

impl<T> Sender<T> {
    /// Try to send a value. It works just like [`mpsc::Sender::send`].
    /// After sending it, it's waking upthe [`mio::poll::Poll`].
    /// 
    /// Note that it does not return any I/O error even if it occurs
    /// when waking up the [`mio::poll::Poll`].
    pub fn send(&self, t: T) -> Result<(), mpsc::SendError<T>> {
        self.tx.send(t)?;

        if let Some(waker) = &mut *self.waker.lock().unwrap() {
            let _ = waker.wake();
        }

        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { waker: self.waker.clone(), tx: self.tx.clone() }
    }
}

/// A wrapper of the [`mpsc::SyncSender`].
pub struct SyncSender<T> {
    waker: Arc<Mutex<Option<Waker>>>,
    tx: mpsc::SyncSender<T>
}

impl<T> SyncSender<T> {
    /// Try to send a value. It works just like [`mpsc::SyncSender::send`].
    /// After sending it, it's waking up the [`mio::poll::Poll`].
    ///
    /// Note that it does not return any I/O error even if it occurs
    /// when waking up the [`mio::poll::Poll`].
    pub fn send(&self, t: T) -> Result<(), mpsc::SendError<T>> {
        self.tx.send(t)?;

        if let Some(waker) = &mut *self.waker.lock().unwrap() {
            let _ = waker.wake();
        }

        Ok(())
    }

    /// Try to send a value. It works just like [`mpsc::SyncSender::try_send`].
    /// After sending it, it's waking up the [`mio::poll::Poll`].
    ///
    /// Note that it does not return any I/O error even if it occurs
    /// when waking up the [`mio::poll::Poll`].
    pub fn try_send(&self, t: T) -> Result<(), mpsc::TrySendError<T>> {
        self.tx.try_send(t)?;

        if let Some(waker) = &mut *self.waker.lock().unwrap() {
            let _ = waker.wake();
        }

        Ok(())
    }
}

impl<T> Clone for SyncSender<T> {
    fn clone(&self) -> Self {
        Self { waker: self.waker.clone(), tx: self.tx.clone() }
    }
}
