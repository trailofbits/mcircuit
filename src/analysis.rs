use std::cmp::{max, min};

use crate::{CombineOperation, HasIO};

/// Generic trait for running something on all the gates in a circuit. Currently used to count wires
pub trait AnalysisPass {
    type Output;

    fn analyze_gate(&mut self, gate: &CombineOperation);

    fn finish_analysis(self) -> Self::Output;

    fn analyze<'a>(circuit: impl Iterator<Item = &'a CombineOperation>) -> Self::Output
    where
        Self: Default,
    {
        let mut result = Self::default();

        for gate in circuit {
            result.analyze_gate(gate);
        }
        result.finish_analysis()
    }
}

pub struct WireCounter {
    largest_arith: usize,
    largest_bool: usize,
    smallest_arith: usize,
    smallest_bool: usize,
}

impl Default for WireCounter {
    fn default() -> Self {
        WireCounter {
            largest_arith: usize::MIN,
            largest_bool: usize::MIN,
            smallest_arith: usize::MAX,
            smallest_bool: usize::MAX,
        }
    }
}

impl AnalysisPass for WireCounter {
    type Output = ((usize, usize), (usize, usize));

    fn analyze_gate(&mut self, gate: &CombineOperation) {
        match gate {
            CombineOperation::GF2(gf2_insn) => {
                for i in gf2_insn.inputs().chain(gf2_insn.outputs()) {
                    self.largest_bool = max(self.largest_bool, i);
                    self.smallest_bool = min(self.smallest_bool, i);
                }
            }
            CombineOperation::Z64(z64_insn) => {
                for i in z64_insn.inputs().chain(z64_insn.outputs()) {
                    self.largest_arith = max(self.largest_arith, i);
                    self.smallest_arith = min(self.smallest_arith, i);
                }
            }
            CombineOperation::B2A(dst, low) => {
                self.largest_arith = max(self.largest_arith, *dst);
                self.largest_bool = max(self.largest_bool, *low + 63);

                self.smallest_arith = min(self.smallest_arith, *dst);
                self.smallest_arith = min(self.smallest_arith, *low);
            }
            CombineOperation::SizeHint(z64, gf2) => {
                self.largest_arith = max(self.largest_arith, *z64);
                self.largest_bool = max(self.largest_bool, *gf2);
            }
        }
    }

    fn finish_analysis(self) -> Self::Output {
        (
            (self.largest_arith + 1, self.largest_bool + 1),
            (self.smallest_arith, self.smallest_bool),
        )
    }
}
