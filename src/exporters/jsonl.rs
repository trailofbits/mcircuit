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
                writeln!(sink, "{}", json!({ i.to_string(): "Input" }))
            }
            Operation::Random(r) => {
                writeln!(sink, "{}", json!({ r.to_string(): "Random" }))
            }
            Operation::Add(o, l, r) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ o.to_string(): "Add", "args": [ l, r ] })
                )
            }
            Operation::AddConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ o.to_string(): "AddConst", "args": [ i, c ] })
                )
            }
            Operation::Sub(o, l, r) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ o.to_string(): "Sub", "args": [ l, r ] })
                )
            }
            Operation::SubConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ o.to_string(): "SubConst", "args": [ i , c ]})
                )
            }
            Operation::Mul(o, l, r) => {
                writeln!(sink, "{}", json!({ o.to_string(): "Mul", "args": [ l, r ]}))
            }
            Operation::MulConst(o, i, c) => {
                writeln!(
                    sink,
                    "{}",
                    json!({ o.to_string(): "MulConst", "args": [ i, c ]})
                )
            }
            Operation::AssertZero(w) => {
                writeln!(sink, "{}", json!({ w.to_string(): "AssertZero"}))
            }
            Operation::Const(w, c) => {
                writeln!(sink, "{}", json!({ w.to_string(): "Const", "args": [ c ]}))
            }
        }
        .and(writeln!(sink, ","))
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
