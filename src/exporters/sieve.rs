//! Export functionality for SIEVE IRs.

use std::io::{Error, ErrorKind, Result, Write};

use crate::exporters::Export;
use crate::Operation;

pub struct IR1;

impl Export<bool> for IR1 {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        match gate {
            Operation::Input(i) => {
                writeln!(sink, "${} <- @short_witness;", i)
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
                writeln!(sink, "${} <- @xor(${}, < {} >);", o, i, *c as u32)
            }
            Operation::Sub(o, l, r) => {
                writeln!(sink, "${} <- @xor(${}, ${});", o, l, r)
            }
            Operation::SubConst(o, i, c) => {
                // NOTE(ww): This could be optimized the way we do for
                // Bristol Fashion: inv when nonzero and just an identity
                // assign when zero.
                writeln!(sink, "${} <- @xor(${}, < {} >);", o, i, *c as u32)
            }
            Operation::Mul(o, l, r) => {
                writeln!(sink, "${} <- @and(${}, ${});", o, l, r)
            }
            Operation::MulConst(o, i, c) => {
                // NOTE(ww): This could be optimized the way we do for
                // Bristol Fashion: inv when zero and just an identity
                // assign when nonzero.
                writeln!(sink, "${} <- @and(${}, < {} >);", o, i, *c as u32)
            }
            Operation::AssertZero(w) => {
                writeln!(sink, "@assert_zero(${});", w)
            }
            Operation::Const(w, c) => {
                writeln!(sink, "${} <- < {} >;", w, *c as u32)
            }
        }
    }

    fn export_circuit(
        gates: &[Operation<bool>],
        witness: &[bool],
        sink: &mut impl Write,
    ) -> Result<()> {
        // Header fields.
        writeln!(sink, "version 1.0.0;")?;
        writeln!(sink, "field characteristic 2 degree 1;")?;

        // Witness body.
        writeln!(sink, "short_witness @begin")?;
        for wit_value in witness.iter() {
            writeln!(sink, "\t< {} >;", *wit_value as u32)?;
        }
        writeln!(sink, "@end")?;

        // We're emitting a boolean circuit, and we don't currently use any special
        // features (like @for, @switch, or @function).
        writeln!(sink, "gate_set: boolean;")?;

        // Circuit body.
        // We're allowed to emit functions in here, before any literal
        // gate directives. But we currently don't need that.
        writeln!(sink, "@begin")?;
        for gate in gates.iter() {
            Self::export_gate(gate, sink)?;
        }
        writeln!(sink, "@end")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::exporters::sieve::IR1;
    use crate::exporters::Export;
    use crate::Operation;

    #[test]
    fn print_example() {
        let mut sink = Vec::new();

        assert!(IR1::export_circuit(
            &[
                Operation::Input(1),
                Operation::Input(2),
                Operation::Input(3),
                Operation::Add(4, 1, 3),
                Operation::Add(5, 2, 3),
                Operation::Mul(6, 5, 4),
                Operation::AddConst(0, 6, true),
                Operation::AssertZero(0)
            ],
            &[false, false, true],
            &mut sink,
        )
        .is_ok());

        let bf = std::str::from_utf8(&sink).unwrap();
        assert_eq!(
            bf,
            "version 1.0.0;
field characteristic 2 degree 1;
short_witness @begin
\t< 0 >;
\t< 0 >;
\t< 1 >;
@end
gate_set: boolean;
@begin
$1 <- @short_witness;
$2 <- @short_witness;
$3 <- @short_witness;
$4 <- @xor($1, $3);
$5 <- @xor($2, $3);
$6 <- @and($5, $4);
$0 <- @xor($6, < 1 >);
@assert_zero($0);
@end
"
        );
    }
}
