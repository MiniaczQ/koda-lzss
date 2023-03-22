use std::ops::{Add, Index};

pub struct IndexOffset<'a, Idx: Add<Output = Idx> + Copy, Output> {
    inner: &'a dyn Index<Idx, Output = Output>,
    offset: Idx,
}

impl<'a, Idx: Add<Output = Idx> + Copy, Output> IndexOffset<'a, Idx, Output> {
    pub fn new(inner: &'a dyn Index<Idx, Output = Output>, offset: Idx) -> Self {
        Self { inner, offset }
    }
}

impl<'a, Idx: Add<Output = Idx> + Copy, Output> Index<Idx> for IndexOffset<'a, Idx, Output> {
    type Output = Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.inner[self.offset + index]
    }
}
