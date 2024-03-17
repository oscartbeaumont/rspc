use futures::{future::Map, stream::FuturesOrdered, FutureExt};
use std::{future::Future, pin::Pin};

/// TODO
pub trait Runtime {
    type Output<T>;

    // fn spawn<F>(f: F);

    // fn finish<T>(self, after: fn(()) -> T) -> Self::Output<T>;
    fn finish<T>(self, after: fn(()) -> T) -> Self::Output<()>;
}

// // TODO: Break out
use tokio::task::{JoinError, JoinHandle};

pub struct TokioRuntime {
    task: Option<JoinHandle<()>>,
}

impl Runtime for TokioRuntime {
    type Output<T> =
        futures::future::Map<tokio::task::JoinHandle<()>, fn(Result<(), JoinError>) -> T>;

    fn finish<T>(self, after: fn(()) -> T) -> Self::Output<()> {
        let test = tokio::spawn(async move { () }); // TODO: Get from `self`
        todo!();
    }
}

pub struct FuturesUnorderedRuntime {
    runtime: FuturesOrdered<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Runtime for FuturesUnorderedRuntime {
    type Output<T> = Map<FuturesOrdered<Pin<Box<dyn Future<Output = ()>>>>, fn() -> T>;

    fn finish<T>(self, after: fn(()) -> T) -> Self::Output<()> {
        todo!();
    }
}

// TODO: FuturesUnordered for batching

// fn testing(
//     f: tokio::task::JoinHandle<()>,
// ) -> futures::future::Map<tokio::task::JoinHandle<()>, fn() -> ()> {
//     // f.map(|a| ())
//     Map::new(f, |a| ())
// }
