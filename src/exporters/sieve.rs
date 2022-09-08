//! Export functionality for SIEVE IRs.

use std::io::{Error, ErrorKind, Result, Write};

use crate::exporters::Export;
use crate::Operation;

pub struct IR1;

impl Export<bool> for IR1 {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        match gate {
            Operation::Input(_) => {
                Err(Error::new(ErrorKind::Other, "can't use input gates in IR1"))
            }
            Operation::Random(_) => {
                // TODO(ww): Is this true?
                Err(Error::new(
                    ErrorKind::Other,
                    "can't use random gates in IR1",
                ))
            }
            Operation::Add(o, l, r) => {
                writeln!(sink, "${} <- @xor(${}, ${});", o, l, r)
            }
            Operation::AddConst(o, i, c) => {
                // NOTE(ww): This could be optimized the way we do for
                // Bristol Fashion: inv when nonzero and just an identity
                // assign when zero.
                writeln!(sink, "${} <- @xor(${}, < {} >);", o, i, c)
            }
            Operation::Sub(o, l, r) => {
                writeln!(sink, "${} <- @xor(${}, ${});", o, l, r)
            }
            Operation::SubConst(o, i, c) => {
                // NOTE(ww): This could be optimized the way we do for
                // Bristol Fashion: inv when nonzero and just an identity
                // assign when zero.
                writeln!(sink, "${} <- @xor(${}, < {} >);", o, i, c)
            }
            Operation::Mul(o, l, r) => {
                writeln!(sink, "${} <- @and(${}, ${});", o, l, r)
            }
            Operation::MulConst(o, i, c) => {
                // NOTE(ww): This could be optimized the way we do for
                // Bristol Fashion: inv when zero and just an identity
                // assign when nonzero.
                writeln!(sink, "${} <- @and(${}, < {} >);", o, i, c)
            }
            Operation::AssertZero(w) => {
                writeln!(sink, "@assert_zero(${});", w)
            }
            Operation::Const(w, c) => {
                writeln!(sink, "${} <- < {} >;", w, c)
            }
            _ => unimplemented!(),
        }
    }

    fn export_circuit(
        gates: &[Operation<bool>],
        witness: &[bool],
        sink: &mut impl Write,
    ) -> Result<()> {
        unimplemented!();
    }
}
