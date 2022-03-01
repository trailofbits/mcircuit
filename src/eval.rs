use crate::analysis::{AnalysisPass, WireCounter};
use crate::parsers::WireHasher;
use crate::{CombineOperation, HasIO, Operation};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};

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
                Operation::AssertZero(src) => {
                    assert!(!bool_wires[src]);
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
                Operation::AssertZero(src) => {
                    assert_eq!(arith_wires[src], 0u64);
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

#[derive(std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash)]
enum ScopeEntry {
    Terminal((String, usize)),
    SubScope(String),
}

#[derive(Clone, Copy)]
enum ScopeType {
    Bool,
    Arith,
}

pub struct VcdDumper {
    writer: BufWriter<File>,
}

impl VcdDumper {
    pub fn for_circuit(
        mut writer: BufWriter<File>,
        circuit: &[CombineOperation],
        bool_hasher: &WireHasher,
        arith_hasher: &WireHasher,
    ) -> Self {
        let mut bool_scopes: HashMap<String, HashSet<ScopeEntry>> = HashMap::new();
        let mut arith_scopes: HashMap<String, HashSet<ScopeEntry>> = HashMap::new();

        for step in circuit {
            match step {
                CombineOperation::GF2(gate) => {
                    for wire in gate.inputs().chain(gate.outputs()) {
                        let backref: String = match bool_hasher.backref(wire) {
                            None => wire.to_string(),
                            Some(s) => s.clone(),
                        };
                        let mut current_scope: &str = "bool_context";

                        let mut scope_tokens = backref.split("::").peekable();
                        while let Some(t) = scope_tokens.next() {
                            if scope_tokens.peek().is_some() {
                                // If this is an intermediate scope
                                bool_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::SubScope(t.into()));
                                current_scope = t;
                            } else {
                                bool_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::Terminal((t.into(), wire)));
                            }
                        }
                    }
                }
                CombineOperation::Z64(gate) => {
                    for wire in gate.inputs().chain(gate.outputs()) {
                        let backref: String = match arith_hasher.backref(wire) {
                            None => wire.to_string(),
                            Some(s) => s.clone(),
                        };
                        let mut current_scope: &str = "arith_context";

                        let mut scope_tokens = backref.split("::").peekable();
                        while let Some(t) = scope_tokens.next() {
                            if scope_tokens.peek().is_some() {
                                // If this is an intermediate scope
                                arith_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::SubScope(t.into()));
                                current_scope = t;
                            } else {
                                arith_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::Terminal((t.into(), wire)));
                            }
                        }
                    }
                }
                CombineOperation::B2A(dst, low) => {
                    let backref: String = match arith_hasher.backref(*dst) {
                        None => dst.to_string(),
                        Some(s) => s.clone(),
                    };
                    let mut current_scope: &str = "b2a_context";

                    let mut scope_tokens = backref.split("::").peekable();
                    while let Some(t) = scope_tokens.next() {
                        if scope_tokens.peek().is_some() {
                            // If this is an intermediate scope
                            arith_scopes
                                .entry(current_scope.into())
                                .or_insert_with(HashSet::new)
                                .insert(ScopeEntry::SubScope(t.into()));
                            current_scope = t;
                        } else {
                            arith_scopes
                                .entry(current_scope.into())
                                .or_insert_with(HashSet::new)
                                .insert(ScopeEntry::Terminal((t.into(), *dst)));
                        }
                    }

                    for wire in *low..*low + 64 {
                        let backref: String = match bool_hasher.backref(wire) {
                            None => wire.to_string(),
                            Some(s) => s.clone(),
                        };
                        let mut current_scope: &str = "b2a_context";

                        let mut scope_tokens = backref.split("::").peekable();
                        while let Some(t) = scope_tokens.next() {
                            if scope_tokens.peek().is_some() {
                                // If this is an intermediate scope
                                bool_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::SubScope(t.into()));
                                current_scope = t;
                            } else {
                                bool_scopes
                                    .entry(current_scope.into())
                                    .or_insert_with(HashSet::new)
                                    .insert(ScopeEntry::Terminal((t.into(), wire)));
                            }
                        }
                    }
                }
                CombineOperation::SizeHint(_, _) => {}
            }
        }

        writer
            .write_all("$version Generated by mcircuit $end\n$timescale 1ns $end\n\n".as_ref())
            .unwrap();
        VcdDumper::write_scope(
            &"bool_context".to_string(),
            ScopeType::Bool,
            &mut writer,
            &bool_scopes,
        )
        .expect("Failed to write Boolean scopes");
        VcdDumper::write_scope(
            &"arith_context".to_string(),
            ScopeType::Arith,
            &mut writer,
            &arith_scopes,
        )
        .expect("Failed to write Arithmetic scopes");
        // VcdDumper::write_scope(
        //     &"b2a_context".to_string(),
        //     ScopeType::Bool,
        //     &mut writer,
        //     &bool_scopes,
        // ).expect("Failed to write boolean B2A scope");
        // VcdDumper::write_scope(
        //     &"b2a_context".to_string(),
        //     ScopeType::Arith,
        //     &mut writer,
        //     &arith_scopes,
        // ).expect("Failed to write arithmetic B2A scope");
        writer
            .write_all("\n$enddefinitions $end\n#0\n$dumpvars\n".as_ref())
            .unwrap();

        VcdDumper { writer }
    }

    fn write_scope(
        scope: &String,
        scope_type: ScopeType,
        writer: &mut BufWriter<File>,
        scopes: &HashMap<String, HashSet<ScopeEntry>>,
    ) -> Result<(), ()> {
        if let Some(current) = scopes.get(scope) {
            writer
                .write_all(format!("$scope module {} $end\n", scope).as_ref())
                .unwrap();

            for entry in current {
                match entry {
                    ScopeEntry::Terminal((label, wire)) => {
                        let (width, prefix) = match scope_type {
                            ScopeType::Bool => (1, "!"),
                            ScopeType::Arith => (64, "@"),
                        };
                        writer
                            .write_all(
                                format!(
                                    "$var wire {} {}{} {} $end\n",
                                    width,
                                    prefix,
                                    wire,
                                    label.replace('[', "(").replace(']', ")")
                                )
                                .as_ref(),
                            )
                            .unwrap();
                    }
                    ScopeEntry::SubScope(sub) => {
                        VcdDumper::write_scope(sub, scope_type, writer, scopes)
                            .expect(&*format!("No scope called {}", sub));
                    }
                }
            }

            writer.write_all("$upscope $end\n".as_ref()).unwrap();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn dump_bool(&mut self, dst: usize, val: bool) {
        self.writer
            .write_all(format!("{}!{}\n", if val { "1" } else { "0" }, dst).as_ref())
            .unwrap();
    }

    pub fn dump_arith(&mut self, dst: usize, val: u64) {
        self.writer
            .write_all(format!("b{:b} @{}\n", val, dst).as_ref())
            .unwrap();
    }

    pub fn finish(&mut self) {
        self.writer.write_all("$end\n#1\n#10\n".as_ref()).unwrap();
        self.writer.flush().unwrap();
    }
}

pub fn dump_vcd(
    program: &[CombineOperation],
    bool_inputs: &[bool],
    arith_inputs: &[u64],
    mut dumper: VcdDumper,
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
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::Random(dst) => {
                    let val: bool = rand::random();
                    bool_wires[dst] = val;
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::Add(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] ^ bool_wires[src2];
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::Sub(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] ^ bool_wires[src2];
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::Mul(dst, src1, src2) => {
                    bool_wires[dst] = bool_wires[src1] & bool_wires[src2];
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::AddConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] ^ c;
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::SubConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] ^ c;
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::MulConst(dst, src, c) => {
                    bool_wires[dst] = bool_wires[src] & c;
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
                Operation::AssertZero(_) => {}
                Operation::Const(dst, c) => {
                    bool_wires[dst] = c;
                    dumper.dump_bool(dst, bool_wires[dst]);
                }
            },
            CombineOperation::Z64(z64_insn) => match *z64_insn {
                Operation::Input(dst) => {
                    arith_wires[dst] = arith_inputs.next().expect("Ran out of arithmetic inputs");
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::Random(dst) => {
                    let val: u64 = rand::random();
                    arith_wires[dst] = val;
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::Add(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_add(arith_wires[src2]);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::Sub(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_sub(arith_wires[src2]);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::Mul(dst, src1, src2) => {
                    arith_wires[dst] = arith_wires[src1].wrapping_mul(arith_wires[src2]);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::AddConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_add(c);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::SubConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_sub(c);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::MulConst(dst, src, c) => {
                    arith_wires[dst] = arith_wires[src].wrapping_mul(c);
                    dumper.dump_arith(dst, arith_wires[dst]);
                }
                Operation::AssertZero(_src) => {}
                Operation::Const(dst, c) => {
                    arith_wires[dst] = c;
                    dumper.dump_arith(dst, arith_wires[dst]);
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
                dumper.dump_arith(*dst, arith_wires[*dst]);
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
    dumper.finish();
}

pub fn largest_wires(program: &[CombineOperation]) -> (usize, usize) {
    if let CombineOperation::SizeHint(z64_cells, gf2_cells) = program[0] {
        (z64_cells, gf2_cells)
    } else {
        WireCounter::default().analyze(program).0
    }
}

pub fn smallest_wires(program: &[CombineOperation]) -> (usize, usize) {
    WireCounter::default().analyze(program).1
}
