use crate::{CombineOperation, Operation, WireValue};

pub struct InputIterator<'a, T> {
    op: &'a T,
    index: usize,
}

impl<'a, T> InputIterator<'a, T> {
    pub(crate) fn new(op: &'a T) -> Self {
        InputIterator { op, index: 0 }
    }
}

pub struct OutputIterator<'a, T> {
    op: &'a T,
    index: usize,
}

impl<'a, T> OutputIterator<'a, T> {
    pub(crate) fn new(op: &'a T) -> Self {
        OutputIterator { op, index: 0 }
    }
}

impl<'a, T: WireValue> Iterator for InputIterator<'a, Operation<T>> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match *self.op {
            Operation::Input(_) => None,
            Operation::Random(_) => None,
            Operation::Sub(_, a, b) => {
                if self.index == 0 {
                    Some(a)
                } else if self.index == 1 {
                    Some(b)
                } else {
                    None
                }
            }
            Operation::SubConst(_, a, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Add(_, a, b) => {
                if self.index == 0 {
                    Some(a)
                } else if self.index == 1 {
                    Some(b)
                } else {
                    None
                }
            }
            Operation::AddConst(_, a, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Mul(_, a, b) => {
                if self.index == 0 {
                    Some(a)
                } else if self.index == 1 {
                    Some(b)
                } else {
                    None
                }
            }
            Operation::MulConst(_, a, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::AssertZero(a) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Const(_, _) => None,
        };
        self.index += 1;
        res
    }
}

impl<'a, T: WireValue> Iterator for OutputIterator<'a, Operation<T>> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match *self.op {
            Operation::Input(a) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Random(a) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Sub(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::SubConst(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Add(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::AddConst(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::Mul(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::MulConst(a, _, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
            Operation::AssertZero(_) => None,
            Operation::Const(a, _) => {
                if self.index == 0 {
                    Some(a)
                } else {
                    None
                }
            }
        };
        self.index += 1;
        res
    }
}

impl<'a> Iterator for InputIterator<'a, CombineOperation> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.op {
            CombineOperation::GF2(op) => InputIterator::new(op).nth(self.index),
            CombineOperation::Z64(op) => InputIterator::new(op).nth(self.index),
            CombineOperation::B2A(_, base) => {
                if self.index < 64 {
                    Some(base + self.index)
                } else {
                    None
                }
            }
            CombineOperation::SizeHint(_, _) => None,
        };
        self.index += 1;
        res
    }
}

impl<'a> Iterator for OutputIterator<'a, CombineOperation> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.op {
            CombineOperation::GF2(op) => OutputIterator::new(op).nth(self.index),
            CombineOperation::Z64(op) => OutputIterator::new(op).nth(self.index),
            CombineOperation::B2A(a, _) => {
                if self.index == 0 {
                    Some(*a)
                } else {
                    None
                }
            }
            CombineOperation::SizeHint(_, _) => None,
        };
        self.index += 1;
        res
    }
}
