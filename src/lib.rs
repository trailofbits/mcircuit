#[macro_use]
extern crate variant_count;

mod io_extractors;
mod tests;

use crate::io_extractors::{InputIterator, OutputIterator};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait WireValue: Copy + PartialEq + std::fmt::Debug {}
impl WireValue for bool {}
impl WireValue for u64 {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, VariantCount)]
pub enum Operation<T: WireValue> {
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

#[derive(Clone, Copy)]
enum OpType<T: WireValue> {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

impl<T: WireValue> Operation<T> {
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

    fn construct<I1, I2>(
        ty: OpType<T>,
        mut inputs: I1,
        mut outputs: I2,
        constant: Option<T>,
    ) -> Operation<T>
    where
        I1: Iterator<Item = usize>,
        I2: Iterator<Item = usize>,
    {
        match ty {
            OpType::InputOp(op) => op(outputs.next().expect("InputOp requires an output wire")),
            OpType::InputConstOp(op) => op(
                outputs
                    .next()
                    .expect("InputConstOp requires an output wire"),
                constant.expect("InputConstOp requires a constant operand"),
            ),
            OpType::OutputConstOp(op) => op(
                inputs.next().expect("OutputConstOp requires an input wire"),
                constant.expect("OutputConstOp requires a constant operand"),
            ),
            OpType::BinaryOp(op) => op(
                outputs.next().expect("BinaryOp requires an output wire"),
                inputs.next().expect("BinaryOp requires two input wires"),
                inputs.next().expect("BinaryOp requires two input wires"),
            ),
            OpType::BinaryConstOp(op) => op(
                outputs
                    .next()
                    .expect("BinaryConstOp requires an output wire"),
                inputs.next().expect("BinaryConstOp requires an input wire"),
                constant.expect("BinaryConstOp requires a constant operand"),
            ),
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
    fn translate<I1, I2>(&self, win: I1, wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = usize>,
        I2: Iterator<Item = usize>;

    fn translate_from_hashmap<'a>(
        &'a self,
        translation_table: HashMap<usize, usize>,
    ) -> Option<Self>
    where
        Self: Sized + HasIO,
        InputIterator<'a, Self>: Iterator<Item = usize>,
        OutputIterator<'a, Self>: Iterator<Item = usize>,
    {
        self.translate(
            self.inputs()
                .map(|x| *translation_table.get(&x).unwrap_or(&x)),
            self.outputs()
                .map(|x| *translation_table.get(&x).unwrap_or(&x)),
        )
    }

    fn translate_from_fn<'a>(
        &'a self,
        input_mapper: fn(usize) -> usize,
        output_mapper: fn(usize) -> usize,
    ) -> Option<Self>
    where
        Self: Sized + HasIO,
        InputIterator<'a, Self>: Iterator<Item = usize>,
        OutputIterator<'a, Self>: Iterator<Item = usize>,
    {
        self.translate(
            self.inputs().map(input_mapper),
            self.outputs().map(output_mapper),
        )
    }
}

impl<T: WireValue> HasIO for Operation<T> {
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

impl<T: WireValue> Translatable for Operation<T> {
    fn translate<'a, I1, I2>(&self, win: I1, wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = usize>,
        I2: Iterator<Item = usize>,
    {
        match self {
            Operation::Input(_) => Some(Operation::<T>::construct(
                OpType::InputOp(Operation::Input),
                win,
                wout,
                None,
            )),
            Operation::Random(_) => Some(Operation::<T>::construct(
                OpType::InputOp(Operation::Random),
                win,
                wout,
                None,
            )),
            Operation::Add(_, _, _) => Some(Operation::<T>::construct(
                OpType::BinaryOp(Operation::Add),
                win,
                wout,
                None,
            )),
            Operation::AddConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConstOp(Operation::AddConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::Sub(_, _, _) => Some(Operation::<T>::construct(
                OpType::BinaryOp(Operation::Sub),
                win,
                wout,
                None,
            )),
            Operation::SubConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConstOp(Operation::SubConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::Mul(_, _, _) => Some(Operation::<T>::construct(
                OpType::BinaryOp(Operation::Mul),
                win,
                wout,
                None,
            )),
            Operation::MulConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConstOp(Operation::MulConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::AssertConst(_, c) => Some(Operation::<T>::construct(
                OpType::OutputConstOp(Operation::AssertConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::Const(_, c) => Some(Operation::<T>::construct(
                OpType::InputConstOp(Operation::Const),
                win,
                wout,
                Some(*c),
            )),
        }
    }
}

impl Translatable for CombineOperation {
    fn translate<'a, I1, I2>(&self, mut win: I1, mut wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = usize>,
        I2: Iterator<Item = usize>,
    {
        match self {
            CombineOperation::GF2(op) => Some(CombineOperation::GF2(
                op.translate(win, wout)
                    .expect("Could not translate underlying GF2 gate"),
            )),
            CombineOperation::Z64(op) => Some(CombineOperation::Z64(
                op.translate(win, wout)
                    .expect("Could not translate underlying Z64 gate"),
            )),
            CombineOperation::B2A(_z64, _gf2) => Some(CombineOperation::B2A(
                wout.next().expect("B2A needs a Z64 output"),
                win.next().expect("B2A needs a GF2 input"),
            )),
            CombineOperation::SizeHint(_z64, _gf2) => None,
        }
    }
}

impl From<Operation<bool>> for CombineOperation {
    fn from(op: Operation<bool>) -> Self {
        CombineOperation::GF2(op)
    }
}

impl From<Operation<u64>> for CombineOperation {
    fn from(op: Operation<u64>) -> Self {
        CombineOperation::Z64(op)
    }
}

impl<T: WireValue> Distribution<Operation<T>> for Standard
where
    Standard: Distribution<(usize, usize, usize, T)>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operation<T> {
        let (out, i0, i1, c): (usize, usize, usize, T) = rand::random();
        Operation::<T>::construct(
            Operation::<T>::random_variant(rng),
            [i0, i1].iter().copied(),
            [out].iter().copied(),
            Some(c),
        )
    }
}
