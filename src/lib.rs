extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate lazy_static;

use futures::Future;
use futures_cpupool::CpuPool;

pub use futures_cpupool::CpuFuture;

lazy_static! {
    static ref THREAD_POOL: CpuPool = { CpuPool::new_num_cpus() };
}


pub trait Spawn {
    type Item;
    type Error;
    fn spawn(self) -> CpuFuture<Self::Item, Self::Error>;
}

impl<F> Spawn for F where
        F: Future + Send + 'static,
        F::Item: Send + 'static,
        F::Error: Send + 'static
{
    type Item = F::Item;
    type Error = F::Error;
    fn spawn(self) -> CpuFuture<Self::Item, Self::Error>  {
        THREAD_POOL.spawn(self)
    }
}