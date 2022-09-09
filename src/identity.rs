use num_traits::{One, Zero};

use crate::{CombineOperation, Operation};

pub trait Identity<T> {
    /*! Trait related to buffer gates. If the gate doesn't change its input value (ie adding zero,
    multiplying by one), then we say this is an "identity" gate, eligible to be folded out. !*/

    fn is_identity(&self) -> bool;

    /// Used to produce an identity gate on the current field when needed.
    fn identity(w_out: Wire, w_in: Wire) -> Self;
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

impl Identity<u64> for Operation<u64> {
    fn is_identity(&self) -> bool {
        match self {
            Operation::AddConst(_, _, c) => c.is_zero(),
            Operation::SubConst(_, _, c) => c.is_zero(),
            Operation::MulConst(_, _, c) => c.is_one(),
            _ => false,
        }
    }

    fn identity(w_out: Wire, w_in: Wire) -> Self {
        Self::AddConst(w_out, w_in, 0u64)
    }
}

impl Identity<bool> for Operation<bool> {
    fn is_identity(&self) -> bool {
        match *self {
            Operation::AddConst(_, _, c) => !c,
            Operation::SubConst(_, _, c) => !c,
            Operation::MulConst(_, _, c) => c,
            _ => false,
        }
    }

    fn identity(w_out: Wire, w_in: Wire) -> Self {
        Self::AddConst(w_out, w_in, false)
    }
}

impl Identity<u64> for CombineOperation {
    fn is_identity(&self) -> bool {
        match self {
            Self::Z64(g) => g.is_identity(),
            _ => false,
        }
    }

    fn identity(w_out: Wire, w_in: Wire) -> Self {
        Self::Z64(Operation::identity(w_out, w_in))
    }
}

impl Identity<bool> for CombineOperation {
    fn is_identity(&self) -> bool {
        match self {
            Self::GF2(g) => g.is_identity(),
            _ => false,
        }
    }

    fn identity(w_out: Wire, w_in: Wire) -> Self {
        Self::GF2(Operation::identity(w_out, w_in))
    }
}
