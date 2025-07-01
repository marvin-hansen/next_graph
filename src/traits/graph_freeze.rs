use crate::CsmGraph;

pub trait Freezable<N, W> {
    fn freeze(self) -> CsmGraph<N, W>;
}
