use crate::analysis::{AnalysisPass, WireCounter};
use crate::{CombineOperation, Operation};

// Evaluates a composite program (in the clear)
pub fn evaluate_composite_program(
    program: &[CombineOperation],
    bool_inputs: &[bool],
    arith_inputs: &[u64],
) {
    let (bool_wire_count, arith_wire_count) = largest_wires(program);

    let mut bool_wires = vec![false; bool_wire_count];
    let mut bool_inputs = bool_inputs.iter().cloned();

    let mut arith_wires = vec![0u64; arith_wire_count];
    let mut arith_inputs = arith_inputs.iter().cloned();

    for step in program {
        match step {
            CombineOperation::GF2(gf2_insn) => match *gf2_insn {
                Operation::Input(dst) => {
                    bool_wires[dst] = bool_inputs.next().expect("Ran out of boolean inputs");
                }
                Operation::Random(dst) => {
                    let val: bool = rand::random();
                    bool_wires[dst] = val;
                }
                Operation::Add(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] ^ bool_wires[src2];
                }
                Operation::Sub(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] ^ bool_wires[src2];
                }
                Operation::Mul(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] & bool_wires[src2];
                }
                Operation::AddConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] ^ c;
                }
                Operation::SubConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] ^ c;
                }
                Operation::MulConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] & c;
                }
                Operation::AssertConst(src, c) => {
                    assert_eq!(bool_wires[src], c);
                }
                Operation::Const(dst, c) => {
                    bool_wires[dst] = c;
                }
            },
            CombineOperation::Z64(z64_insn) => match *z64_insn {
                Operation::Input(dst) => {
                    arith_wires[dst] = arith_inputs.next().expect("Ran out of arithmetic inputs");
                }
                Operation::Random(dst) => {
                    let val: u64 = rand::random();
                    arith_wires[dst] = val;
                }
                Operation::Add(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_add(arith_wires[src2]);
                }
                Operation::Sub(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_sub(arith_wires[src2]);
                }
                Operation::Mul(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_mul(arith_wires[src2]);
                }
                Operation::AddConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_add(c);
                }
                Operation::SubConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_sub(c);
                }
                Operation::MulConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_mul(c);
                }
                Operation::AssertConst(src, c) => {
                    assert_eq!(arith_wires[src], c);
                }
                Operation::Const(dst, c) => {
                    arith_wires[dst] = c;
                }
            },
            CombineOperation::B2A(dst, low) => {
                let mut running_val: u64 = 0;
                let mut power: u64 = 1;
                for bit in bool_wires.iter().skip(*low).take(64) {
                    running_val = running_val.wrapping_add(if *bit { power } else { 0 });
                    power = power.wrapping_shl(1);
                }
                arith_wires[*dst] = running_val;
            }
            CombineOperation::SizeHint(z64, gf2) => {
                if bool_wires.len() < *gf2 {
                    bool_wires.resize(*gf2, false);
                }
                if arith_wires.len() < *z64 {
                    arith_wires.resize(*z64, 0);
                }
            }
        }
    }
}

pub fn largest_wires(program: &[CombineOperation]) -> (usize, usize) {
    if let CombineOperation::SizeHint(z64_cells, gf2_cells) = program[0] {
        (z64_cells, gf2_cells)
    } else {
        WireCounter::default().analyze(program)
    }
}
