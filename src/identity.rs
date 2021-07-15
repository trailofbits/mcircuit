use num_traits::{One, Zero};

use crate::{CombineOperation, Operation};

pub trait Identity {
    fn is_identity(&self) -> bool;
}

// /// Rustc won't let us use any generics here because `bool` might implement Zero and One in
// /// the future (though it currently doesn't) and also won't allow us to rectify this problem
// /// ourselves by just implementing it locally, since it's a foreign trait. Cursed.
// impl<T: Zero + One + WireValue> Identity for Operation<T>{
//     fn is_identity(&self) -> bool {
//         match self {
//             Operation::AddConst(_, _, c) => {c.is_zero()}
//             Operation::SubConst(_, _, c) => {c.is_zero()}
//             Operation::MulConst(_, _, c) => {c.is_one()}
//             _ => {false}
//         }
//     }
// }

impl Identity for Operation<u64> {
    fn is_identity(&self) -> bool {
        match self {
            Operation::AddConst(_, _, c) => c.is_zero(),
            Operation::SubConst(_, _, c) => c.is_zero(),
            Operation::MulConst(_, _, c) => c.is_one(),
            _ => false,
        }
    }
}

impl Identity for Operation<bool> {
    fn is_identity(&self) -> bool {
        match *self {
            Operation::AddConst(_, _, c) => !c,
            Operation::SubConst(_, _, c) => !c,
            Operation::MulConst(_, _, c) => c,
            _ => false,
        }
    }
}

impl Identity for CombineOperation {
    fn is_identity(&self) -> bool {
        match self {
            CombineOperation::GF2(g) => g.is_identity(),
            CombineOperation::Z64(g) => g.is_identity(),
            CombineOperation::B2A(_, _) => false,
            CombineOperation::SizeHint(_, _) => false,
        }
    }
}
