use crate::{CombineOperation, Operation};
use std::cmp::max;

pub trait AnalysisPass {
    type Output;

    fn analyze_gate(&mut self, gate: &CombineOperation);

    fn finish_analysis(&self) -> Self::Output;

    fn analyze(&mut self, circuit: &[CombineOperation]) -> Self::Output {
        for gate in circuit {
            self.analyze_gate(gate);
        }
        self.finish_analysis()
    }
}

#[derive(Default)]
pub struct WireCounter {
    arith_wires: usize,
    bool_wires: usize,
}

impl AnalysisPass for WireCounter {
    type Output = (usize, usize);

    fn analyze_gate(&mut self, gate: &CombineOperation) {
        match gate {
            CombineOperation::GF2(gf2_insn) => match *gf2_insn {
                Operation::Input(dst) => {
                    self.bool_wires = max(self.bool_wires, dst);
                }
                Operation::Random(dst) => {
                    self.bool_wires = max(self.bool_wires, dst);
                }
                Operation::Add(dst, src1, src2) => {
                    self.bool_wires = max(self.bool_wires, max(dst, max(src1, src2)));
                }
                Operation::Sub(dst, src1, src2) => {
                    self.bool_wires = max(self.bool_wires, max(dst, max(src1, src2)));
                }
                Operation::Mul(dst, src1, src2) => {
                    self.bool_wires = max(self.bool_wires, max(dst, max(src1, src2)));
                }
                Operation::AddConst(dst, src, _c) => {
                    self.bool_wires = max(self.bool_wires, max(dst, src));
                }
                Operation::SubConst(dst, src, _c) => {
                    self.bool_wires = max(self.bool_wires, max(dst, src));
                }
                Operation::MulConst(dst, src, _c) => {
                    self.bool_wires = max(self.bool_wires, max(dst, src));
                }
                Operation::AssertZero(src) => {
                    self.bool_wires = max(self.bool_wires, src);
                }
                Operation::Const(dst, _c) => {
                    self.bool_wires = max(self.bool_wires, dst);
                }
            },
            CombineOperation::Z64(z64_insn) => match *z64_insn {
                Operation::Input(dst) => {
                    self.arith_wires = max(self.arith_wires, dst);
                }
                Operation::Random(dst) => {
                    self.arith_wires = max(self.arith_wires, dst);
                }
                Operation::Add(dst, src1, src2) => {
                    self.arith_wires = max(self.arith_wires, max(dst, max(src1, src2)));
                }
                Operation::Sub(dst, src1, src2) => {
                    self.arith_wires = max(self.arith_wires, max(dst, max(src1, src2)));
                }
                Operation::Mul(dst, src1, src2) => {
                    self.arith_wires = max(self.arith_wires, max(dst, max(src1, src2)));
                }
                Operation::AddConst(dst, src, _c) => {
                    self.arith_wires = max(self.arith_wires, max(dst, src));
                }
                Operation::SubConst(dst, src, _c) => {
                    self.arith_wires = max(self.arith_wires, max(dst, src));
                }
                Operation::MulConst(dst, src, _c) => {
                    self.arith_wires = max(self.arith_wires, max(dst, src));
                }
                Operation::AssertZero(src) => {
                    self.arith_wires = max(self.arith_wires, src);
                }
                Operation::Const(dst, _c) => {
                    self.arith_wires = max(self.arith_wires, dst);
                }
            },
            CombineOperation::B2A(dst, low) => {
                self.arith_wires = max(self.arith_wires, *dst);
                self.bool_wires = max(self.bool_wires, *low + 63);
            }
            CombineOperation::SizeHint(z64, gf2) => {
                self.arith_wires = max(self.arith_wires, *z64);
                self.bool_wires = max(self.bool_wires, *gf2);
            }
        }
    }

    fn finish_analysis(&self) -> Self::Output {
        (self.arith_wires + 1, self.bool_wires + 1)
    }
}
