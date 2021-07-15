use std::collections::HashMap;

use crate::io_extractors::{InputIterator, OutputIterator};
use crate::{CombineOperation, HasIO, OpType, Operation, WireValue};

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

impl<T: WireValue> Translatable for Operation<T> {
    fn translate<'a, I1, I2>(&self, win: I1, wout: I2) -> Option<Self>
    where
        Self: Sized,
        I1: Iterator<Item = usize>,
        I2: Iterator<Item = usize>,
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
