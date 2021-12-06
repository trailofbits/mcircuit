use crate::{CombineOperation, Operation, WireValue, U256};

pub trait HasConst<T> {
    fn constant(&self) -> Option<T>;
}

impl<T: WireValue> HasConst<T> for Operation<T> {
    fn constant(&self) -> Option<T> {
        match *self {
            Operation::AddConst(_, _, c) => Some(c),
            Operation::SubConst(_, _, c) => Some(c),
            Operation::MulConst(_, _, c) => Some(c),
            Operation::Const(_, c) => Some(c),
            _ => None,
        }
    }
}

impl HasConst<bool> for CombineOperation {
    fn constant(&self) -> Option<bool> {
        match self {
            CombineOperation::GF2(g) => g.constant(),
            _ => None,
        }
    }
}

impl HasConst<u8> for CombineOperation {
    fn constant(&self) -> Option<u8> {
        match self {
            CombineOperation::GF2AsU8(g) => g.constant(),
            _ => None,
        }
    }
}

impl HasConst<u64> for CombineOperation {
    fn constant(&self) -> Option<u64> {
        match self {
            CombineOperation::Z64(g) => g.constant(),
            _ => None,
        }
    }
}

impl HasConst<U256> for CombineOperation {
    fn constant(&self) -> Option<U256> {
        match self {
            CombineOperation::Z256(g) => g.constant(),
            _ => None,
        }
    }
}
