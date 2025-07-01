use crate::{CsmGraph, DynamicGraph};

pub trait Freezable<N, W> {
    fn freeze(self) -> CsmGraph<N, W>;
}

pub trait Unfreezable<N, W> {
    fn unfreeze(self) -> DynamicGraph<N, W>;
}
