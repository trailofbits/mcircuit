mod io_extractors;

use crate::io_extractors::{InputIterator, OutputIterator};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Operation<T> {
    Input(usize),
    Random(usize),
    Add(usize, usize, usize),
    AddConst(usize, usize, T),
    Sub(usize, usize, usize),
    SubConst(usize, usize, T),
    Mul(usize, usize, usize),
    MulConst(usize, usize, T),
    AssertConst(usize, T),
    Const(usize, T),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CombineOperation {
    /// Circuit Operation on GF2 Finite Field
    GF2(Operation<bool>),
    /// Circuit Operation on 64-bit integer ring
    Z64(Operation<u64>),

    /// Converts a value on GF2 to a value on Z64
    /// Takes: (dst, src) where src is the _low bit_ of the 64-bit GF2 slice
    B2A(usize, usize),

    /// Information about the number of wires needed to evaluate the circuit. As with B2A,
    /// first item is Z64, second is GF2.
    SizeHint(usize, usize),
}

pub trait HasIO<T> {
    fn inputs(&self) -> InputIterator<T>;
    fn outputs(&self) -> OutputIterator<T>;
}

impl<T> HasIO<Operation<T>> for Operation<T> {
    #[inline(always)]
    fn inputs(&self) -> InputIterator<Operation<T>> {
        InputIterator::new(self)
    }

    #[inline(always)]
    fn outputs(&self) -> OutputIterator<Operation<T>> {
        OutputIterator::new(self)
    }
}

impl HasIO<CombineOperation> for CombineOperation {
    #[inline(always)]
    fn inputs(&self) -> InputIterator<CombineOperation> {
        InputIterator::new(self)
    }

    #[inline(always)]
    fn outputs(&self) -> OutputIterator<CombineOperation> {
        OutputIterator::new(self)
    }
}
