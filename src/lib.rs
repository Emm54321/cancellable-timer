#![deny(missing_docs)]

//! Crate that implements a timer with a `sleep` method that can be cancelled.
//!
//! # Example
//!
//! ```
//! use std::time::Duration;
//! use cancellable_timer::*;
//!
//! fn main() {
//!     let (mut timer, canceller) = Timer::new2().unwrap();
//!
//!     // Spawn a thread that will cancel the timer after 2s.
//!     std::thread::spawn(move || {
//!         std::thread::sleep(Duration::from_secs(2));
//!         println!("Stop the timer.");
//!         canceller.cancel();
//!     });
//!
//!     println!("Wait 10s");
//!     let r = timer.sleep(Duration::from_secs(10));
//!     println!("Done: {:?}", r);
//! }
//! ```

extern crate mio;

use std::io;
use std::time::Duration;

use mio::*;

/// A timer object that can be used to put the current thread to sleep
/// or to start a callback after a given amount of time.
pub struct Timer {
    poll: Poll,
    token: Token,
    _registration: Registration,
    events: Events,
}

/// An object that allows cancelling the associated [Timer](struct.Timer.html).
#[derive(Clone)]
pub struct Canceller {
    set_readiness: SetReadiness,
}

impl Timer {
    /// Create a [Timer](struct.Timer.html) and its associated [Canceller](struct.Canceller.html).
    pub fn new2() -> io::Result<(Self, Canceller)> {
        let poll = Poll::new()?;

        let token = Token(0);
        let (registration, set_readiness) = Registration::new2();
        poll.register(&registration, token, Ready::readable(), PollOpt::edge())?;

        Ok((
            Timer {
                poll,
                token,
                _registration: registration,
                events: Events::with_capacity(4),
            },
            Canceller { set_readiness },
        ))
    }

    /// Put the current thread to sleep until the given time has
    /// elapsed or the timer is cancelled.
    ///
    /// Returns:
    /// * Ok(()) if the given time has elapsed.
    /// * An [Error](https://docs.rust-lang.org/std/io/struct.Error.html)
    /// of kind [ErrorKind::Interrupted](https://docs.rust-lang.org/std/io/enum.ErrorKind.html)
    /// if the timer has been cancelled.
    /// * Some other [Error](https://docs.rust-lang.org/std/io/struct.Error.html)
    /// if something goes wrong.
    pub fn sleep(&mut self, duration: Duration) -> io::Result<()> {
        self.poll.poll(&mut self.events, Some(duration))?;
        for event in self.events.iter() {
            if event.token() == self.token {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "timer cancelled",
                ));
            }
        }
        Ok(())
    }

    /// Run a callback on a new thread after a specified amount of time.
    /// The callback is not run if `after` returns an error.
    ///
    /// Otherwise, the callback is given:
    /// * Ok(()) if the amount of time has elapsed.
    /// * An [Error](https://docs.rust-lang.org/std/io/struct.Error.html)
    /// of kind [ErrorKind::Interrupted](https://docs.rust-lang.org/std/io/enum.ErrorKind.html)
    /// if the timer has been cancelled.
    /// * Some other [Error](https://docs.rust-lang.org/std/io/struct.Error.html)
    /// if something goes wrong.
    pub fn after<F>(wait: Duration, callback: F) -> io::Result<Canceller>
    where
        F: FnOnce(io::Result<()>),
        F: Send + 'static,
    {
        let (mut timer, canceller) = Timer::new2()?;
        std::thread::Builder::new().spawn(move || {
            callback(timer.sleep(wait));
        })?;
        Ok(canceller)
    }
}

impl Canceller {
    /// Cancel the associated [Timer](struct.Timer.html).
    pub fn cancel(&self) -> io::Result<()> {
        self.set_readiness.set_readiness(Ready::readable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn uninterrupted_sleep() {
        let (mut timer, _) = Timer::new2().unwrap();
        let r = timer.sleep(Duration::from_secs(1));
        assert!(r.is_ok());
    }

    #[test]
    fn cancel_before_sleep() {
        let (mut timer, canceller) = Timer::new2().unwrap();
        canceller.cancel().unwrap();
        let r = timer.sleep(Duration::from_secs(1));
        assert_eq!(r.unwrap_err().kind(), io::ErrorKind::Interrupted);
    }

    #[test]
    fn cancel_during_sleep() {
        let (mut timer, canceller) = Timer::new2().unwrap();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2));
            canceller.cancel().unwrap();
        });
        let r = timer.sleep(Duration::from_secs(10));
        assert_eq!(r.unwrap_err().kind(), io::ErrorKind::Interrupted);
    }
}
