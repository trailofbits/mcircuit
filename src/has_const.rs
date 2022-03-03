use crate::{CombineOperation, Operation, WireValue};

pub trait HasConst<T> {
    /// For gates that include constant data (AddConst, MulConst, etc) provides a method to access
    /// the constant. Returns `None` for all other types of gates.
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

impl HasConst<u64> for CombineOperation {
    fn constant(&self) -> Option<u64> {
        match self {
            CombineOperation::Z64(g) => g.constant(),
            _ => None,
        }
    }
}
