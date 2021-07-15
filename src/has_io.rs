use crate::io_extractors::{InputIterator, OutputIterator};
use crate::{CombineOperation, Operation, WireValue};

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

pub trait HasIO {
    fn inputs(&self) -> InputIterator<Self>
    where
        Self: Sized;
    fn outputs(&self) -> OutputIterator<Self>
    where
        Self: Sized;

    fn dst<'a>(&'a self) -> Option<usize>
    where
        Self: 'a + Sized,
        OutputIterator<'a, Self>: Iterator<Item = usize>,
    {
        self.outputs().next()
    }
}
