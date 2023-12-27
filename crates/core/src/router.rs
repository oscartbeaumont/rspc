use crate::{internal, Executor};

pub trait Router: internal::SealedRouter {
    type Ctx;

    fn build(self) -> Executor;
}
