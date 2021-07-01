#[macro_use]
extern crate variant_count;

mod io_extractors;
mod tests;

use crate::io_extractors::{InputIterator, OutputIterator};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, VariantCount)]
pub enum Operation<T> {
    /// Read a value from input and emit it on the wire
    Input(usize),
    /// Emit a random value on the wire
    Random(usize),
    Add(usize, usize, usize),
    AddConst(usize, usize, T),
    /// Subtract the final wire from the second wire
    Sub(usize, usize, usize),
    SubConst(usize, usize, T),
    Mul(usize, usize, usize),
    MulConst(usize, usize, T),
    /// Assert that the wire has the const value
    AssertConst(usize, T),
    /// Emit the const value on the wire
    Const(usize, T),
}

enum OpType<T> {
    /// (dst)
    InputOp(fn(usize) -> Operation<T>),
    /// (dst, constant)
    InputConstOp(fn(usize, T) -> Operation<T>),
    /// (src, constant)
    OutputConstOp(fn(usize, T) -> Operation<T>),
    /// (dst, src1, src2)
    BinaryOp(fn(usize, usize, usize) -> Operation<T>),
    /// (dst, src, constant)
    BinaryConstOp(fn(usize, usize, T) -> Operation<T>),
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

impl<T> Operation<T> {
    fn random_variant<R: Rng + ?Sized>(rng: &mut R) -> OpType<T> {
        match rng.gen_range(0..Operation::<T>::VARIANT_COUNT) {
            0 => OpType::InputOp(Operation::Input),
            1 => OpType::InputOp(Operation::Random),
            2 => OpType::BinaryOp(Operation::Add),
            3 => OpType::BinaryConstOp(Operation::AddConst),
            4 => OpType::BinaryOp(Operation::Sub),
            5 => OpType::BinaryConstOp(Operation::SubConst),
            6 => OpType::BinaryOp(Operation::Mul),
            7 => OpType::BinaryConstOp(Operation::MulConst),
            8 => OpType::OutputConstOp(Operation::AssertConst),
            9 => OpType::InputConstOp(Operation::Const),
            _ => {
                unimplemented!("Operation.random_variant is missing some variants")
            }
        }
    }
}

pub trait HasIO {
    fn inputs(&self) -> InputIterator<Self>
    where
        Self: Sized;
    fn outputs(&self) -> OutputIterator<Self>
    where
        Self: Sized;
}

pub trait Translatable {
    fn translate(&self, win: &[usize], wout: &[usize]) -> Option<Self>
    where
        Self: Sized;
}

impl<T> HasIO for Operation<T> {
    #[inline(always)]
    fn inputs(&self) -> InputIterator<Operation<T>> {
        InputIterator::new(self)
    }

    #[inline(always)]
    fn outputs(&self) -> OutputIterator<Operation<T>> {
        OutputIterator::new(self)
    }
}

impl HasIO for CombineOperation {
    #[inline(always)]
    fn inputs(&self) -> InputIterator<CombineOperation> {
        InputIterator::new(self)
    }

    #[inline(always)]
    fn outputs(&self) -> OutputIterator<CombineOperation> {
        OutputIterator::new(self)
    }
}
