extern crate futures;
extern crate futures_cpupool;
extern crate tokio_timer;
#[macro_use]
extern crate lazy_static;

use futures::{Future, Poll, Async};
use futures_cpupool::CpuPool;
use std::time::Duration;
use tokio_timer::{Timer, Sleep, TimerError};

pub use futures_cpupool::CpuFuture;

lazy_static! {
    static ref THREAD_POOL: CpuPool = { CpuPool::new_num_cpus() };
}

pub enum Error<E> {
    FutureError(E),
    TimedOut,
    TimerFailed(TimerError)
}


#[derive(Debug)]
pub struct Timeout<T> {
    future: Option<T>,
    sleep: Sleep,
}

impl<F> Future for Timeout<F>
    where F: Future,
{
    type Item = F::Item;
    type Error = Error<F::Error>;

    fn poll(&mut self) -> Poll<F::Item, Self::Error> {
        // First, try polling the future
        match self.future {
            Some(ref mut f) => {
                match f.poll() {
                    Ok(Async::NotReady) => {}
                    Err(e) => return Err(Error::FutureError(e)),
                    Ok(Async::Ready(v)) => return Ok(Async::Ready(v)),
                }
            }
            None => panic!("cannot call poll once value is consumed"),
        }

        // Now check the timer
        match self.sleep.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_)) => {
                // Timeout has elapsed, error the future
                self.future.take().unwrap();
                Err(Error::TimedOut)
            }
            Err(e) => {
                // Something went wrong with the underlying timeout
                self.future.take().unwrap();
                Err(Error::TimerFailed(e))
            }
        }
    }
}


pub trait FutureExt {
    type Item;
    type Error;
    fn spawn(self) -> CpuFuture<Self::Item, Self::Error>;
    fn wait_for(self, duration: Duration) -> Result<Self::Item, Error<Self::Error>>;
}

impl<F> FutureExt for F where
        F: Future + Send + 'static,
        F::Item: Send + 'static,
        F::Error: Send + 'static
{
    type Item = F::Item;
    type Error = F::Error;
    fn spawn(self) -> CpuFuture<Self::Item, Self::Error>  {
        THREAD_POOL.spawn(self)
    }

    fn wait_for(self, duration: Duration) -> Result<Self::Item, Error<Self::Error>> {
        let timer = Timer::default();
        let sleep = timer.sleep(duration);
        let timeout = Timeout { future: Some(self), sleep: sleep };
        timeout.wait()
    }
}