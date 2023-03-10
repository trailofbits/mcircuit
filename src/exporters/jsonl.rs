//! Export functionality for JSON.

use serde_json::json;
use std::io::{Result, Write};

use crate::exporters::Export;
use crate::Operation;

pub struct JSONL;

impl Export<bool> for JSONL {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        match gate {
            Operation::Input(i) => {
                writeln!(sink, "{}", json!({ "wire": i, "op": "Input", "args": [] }))
            }
            Operation::Random(r) => {
                writeln!(sink, "{}", json!({ "wire": r, "op": "Random", "args": [] }))
            }
            Operation::Add(o, l, r) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "Add", "args": [ l, r ] })
                )
            }
            Operation::AddConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "AddConst", "args": [ i, c ] })
                )
            }
            Operation::Sub(o, l, r) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "Sub", "args": [ l, r ] })
                )
            }
            Operation::SubConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "SubConst", "args": [ i , c ] })
                )
            }
            Operation::Mul(o, l, r) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "Mul", "args": [ l, r ] })
                )
            }
            Operation::MulConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": o, "op": "MulConst", "args": [ i, c ] })
                )
            }
            Operation::AssertZero(w) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": w, "op": "AssertZero", "args": [] })
                )
            }
            Operation::Const(w, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ "wire": w, "op": "Const", "args": [ c ] })
                )
            }
        }
    }

    fn export_circuit(
        gates: &[Operation<bool>],
        _witness: &[bool],
        sink: &mut impl Write,
    ) -> Result<()> {
        for gate in gates.iter() {
            Self::export_gate(gate, sink)?;
        }

        Ok(())
    }
}
