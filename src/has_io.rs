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
    //! Applies to all gates, allows access to the input and output wire IDs of the gates

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
        //! For convenience, allows access to the (optional) output, since each gate only
        //! ever has one at most.
        self.outputs().next()
    }
}
