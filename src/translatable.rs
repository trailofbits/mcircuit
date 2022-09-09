use std::collections::HashMap;

use crate::Wire;
use crate::io_extractors::{InputIterator, OutputIterator};
use crate::{CombineOperation, HasIO, OpType, Operation, WireValue};

/// Defines a number of helper methods for replacing the I/O wires on a gate with new ones
pub trait Translatable {
    /// takes an iterator of input wires and an iterator of output wires, and creates a new gate
    /// of the same type using the inputs and outputs. The current input and output wires have
    /// no bearing on the new wires, just the gate type.
    fn translate<I1, I2>(&self, win: I1, wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = Wire>,
        I2: Iterator<Item = Wire>;

    /// Takes a hashmap, and looks for existing wires in the keys. Replaces any existing wire keys
    /// with the value from the hashmap.
    fn translate_from_hashmap<'a>(
        &'a self,
        translation_table: HashMap<usize, Wire>,
    ) -> Option<Self>
    where
        Self: Sized + HasIO,
        InputIterator<'a, Self>: Iterator<Item = Wire>,
        OutputIterator<'a, Self>: Iterator<Item = Wire>,
    {
        self.translate(
            self.inputs()
                .map(|x| *translation_table.get(&x).unwrap_or(&x)),
            self.outputs()
                .map(|x| *translation_table.get(&x).unwrap_or(&x)),
        )
    }

    /// Calls a function on the I/O wires and replaces them with the output of the function.
    fn translate_from_fn<'a>(
        &'a self,
        input_mapper: fn(Wire) -> usize,
        output_mapper: fn(Wire) -> usize,
    ) -> Option<Self>
    where
        Self: Sized + HasIO,
        InputIterator<'a, Self>: Iterator<Item = Wire>,
        OutputIterator<'a, Self>: Iterator<Item = Wire>,
    {
        self.translate(
            self.inputs().map(input_mapper),
            self.outputs().map(output_mapper),
        )
    }
}

impl<T: WireValue> Translatable for Operation<T> {
    fn translate<'a, I1, I2>(&self, win: I1, wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = Wire>,
        I2: Iterator<Item = Wire>,
    {
        match self {
            Operation::Input(_) => Some(Operation::<T>::construct(
                OpType::Input(Operation::Input),
                win,
                wout,
                None,
            )),
            Operation::Random(_) => Some(Operation::<T>::construct(
                OpType::Input(Operation::Random),
                win,
                wout,
                None,
            )),
            Operation::Add(_, _, _) => Some(Operation::<T>::construct(
                OpType::Binary(Operation::Add),
                win,
                wout,
                None,
            )),
            Operation::AddConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConst(Operation::AddConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::Sub(_, _, _) => Some(Operation::<T>::construct(
                OpType::Binary(Operation::Sub),
                win,
                wout,
                None,
            )),
            Operation::SubConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConst(Operation::SubConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::Mul(_, _, _) => Some(Operation::<T>::construct(
                OpType::Binary(Operation::Mul),
                win,
                wout,
                None,
            )),
            Operation::MulConst(_, _, c) => Some(Operation::<T>::construct(
                OpType::BinaryConst(Operation::MulConst),
                win,
                wout,
                Some(*c),
            )),
            Operation::AssertZero(_) => Some(Operation::<T>::construct(
                OpType::Output(Operation::AssertZero),
                win,
                wout,
                None,
            )),
            Operation::Const(_, c) => Some(Operation::<T>::construct(
                OpType::InputConst(Operation::Const),
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
        I1: Iterator<Item = Wire>,
        I2: Iterator<Item = Wire>,
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
