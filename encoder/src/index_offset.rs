use std::ops::{Add, Index};

pub struct IndexMapper<'a, Idx: Add<Output = Idx> + Copy, Output> {
    inner: &'a dyn Index<Idx, Output = Output>,
    offset: Idx,
}

impl<'a, Idx: Add<Output = Idx> + Copy, Output> IndexMapper<'a, Idx, Output> {
    pub fn new(inner: &'a dyn Index<Idx, Output = Output>, offset: Idx) -> Self {
        Self { inner, offset }
    }
}

impl<'a, Idx: Add<Output = Idx> + Copy, Output> Index<Idx> for IndexMapper<'a, Idx, Output> {
    type Output = Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.inner[self.offset + index]
    }
}
