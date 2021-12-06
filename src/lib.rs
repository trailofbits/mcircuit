// Rustc tried to guide me back to the path of righteousness, but I have been
// tempted by the forbidden `fn(&self) -> [u8; size_of::<Self>()]`
#![feature(generic_const_exprs)]

#[macro_use]
extern crate variant_count;

use num_traits::Zero;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use uint::construct_uint;

pub use eval::{evaluate_composite_program, largest_wires, smallest_wires};
pub use has_const::HasConst;
pub use has_io::HasIO;
pub use identity::Identity;
pub use parsers::Parse;
pub use translatable::Translatable;

mod analysis;
mod eval;
pub mod exporters;
mod has_const;
mod has_io;
mod identity;
mod io_extractors;
pub mod parsers;
mod tests;
mod translatable;

// U256 consisting of 4 x 64-bit words
construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}

pub trait WireValue: Copy + PartialEq + std::fmt::Debug + Serialize + Sized {
    fn is_zero(&self) -> bool;

    fn to_le_bytes(&self) -> [u8; size_of::<Self>()];
}

impl WireValue for bool {
    fn is_zero(&self) -> bool {
        !*self
    }

    fn to_le_bytes(&self) -> [u8; 1] {
        [if *self { 1u8 } else { 0u8 }]
    }
}

impl WireValue for u64 {
    fn is_zero(&self) -> bool {
        Zero::is_zero(self)
    }

    fn to_le_bytes(&self) -> [u8; 8] {
        u64::to_le_bytes(*self)
    }
}

impl WireValue for u8 {
    fn is_zero(&self) -> bool {
        Zero::is_zero(self)
    }

    fn to_le_bytes(&self) -> [u8; 1] {
        u8::to_le_bytes(*self)
    }
}

impl WireValue for U256 {
    fn is_zero(&self) -> bool {
        *self == U256::from(0)
    }

    fn to_le_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        bytes
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, VariantCount)]
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
    /// Assert that the wire has the const value zero
    AssertZero(usize),
    /// Emit the const value on the wire
    Const(usize, T),
}

#[derive(Clone, Copy)]
enum OpType<T: WireValue> {
    /// (dst)
    Input(fn(usize) -> Operation<T>),
    /// (dst, constant)
    InputConst(fn(usize, T) -> Operation<T>),
    /// (src, constant)
    Output(fn(usize) -> Operation<T>),
    /// (dst, src1, src2)
    Binary(fn(usize, usize, usize) -> Operation<T>),
    /// (dst, src, constant)
    BinaryConst(fn(usize, usize, T) -> Operation<T>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CombineOperation {
    /// Circuit Operation on GF2 Finite Field
    GF2(Operation<bool>),
    /// Circuit Operation on 8-bit integer ring. Treated as _BOOLEAN_ operations that carry extra
    /// bits so we can insert new values to indicate error states.
    GF2AsU8(Operation<u8>),
    /// Circuit Operation on 64-bit integer ring
    Z64(Operation<u64>),
    /// Circuit operation on 256-bit integer ring (Boxed to reduce enum size)
    Z256(Box<Operation<U256>>),

    /// Converts a value on GF2 to a value on Z64
    /// Takes: (dst, src) where src is the _low bit_ of the 64-bit GF2 slice
    /// This means that the least significant bit of the Z64 value will come from the
    /// GF2 wire with the lowest index. Make sure your circuits are designed accordingly.
    B2A(usize, usize),

    /// Information about the number of wires needed to evaluate the circuit. As with B2A,
    /// first item is Z64, second is GF2.
    SizeHint(usize, usize),
}

impl<T: WireValue> Operation<T> {
    fn random_variant<R: Rng + ?Sized>(rng: &mut R) -> OpType<T> {
        match rng.gen_range(0..Operation::<T>::VARIANT_COUNT) {
            0 => OpType::Input(Operation::Input),
            1 => OpType::Input(Operation::Random),
            2 => OpType::Binary(Operation::Add),
            3 => OpType::BinaryConst(Operation::AddConst),
            4 => OpType::Binary(Operation::Sub),
            5 => OpType::BinaryConst(Operation::SubConst),
            6 => OpType::Binary(Operation::Mul),
            7 => OpType::BinaryConst(Operation::MulConst),
            8 => OpType::Output(Operation::AssertZero),
            9 => OpType::InputConst(Operation::Const),
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
            OpType::Input(op) => op(outputs.next().expect("Input op requires an output wire")),
            OpType::InputConst(op) => op(
                outputs
                    .next()
                    .expect("InputConst op requires an output wire"),
                constant.expect("InputConst op requires a constant operand"),
            ),
            OpType::Output(op) => op(inputs.next().expect("Output op requires an input wire")),
            OpType::Binary(op) => op(
                outputs.next().expect("Binary op requires an output wire"),
                inputs.next().expect("Binary op requires two input wires"),
                inputs.next().expect("Binary op requires two input wires"),
            ),
            OpType::BinaryConst(op) => op(
                outputs
                    .next()
                    .expect("BinaryConst op requires an output wire"),
                inputs
                    .next()
                    .expect("BinaryConst op requires an input wire"),
                constant.expect("BinaryConst op requires a constant operand"),
            ),
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

pub trait Gate<T>: HasIO + HasConst<T> + Translatable + Identity<T> {}
impl Gate<u64> for Operation<u64> {}
impl Gate<bool> for Operation<bool> {}
impl<T: WireValue> Gate<T> for CombineOperation where CombineOperation: HasConst<T> + Identity<T> {}
