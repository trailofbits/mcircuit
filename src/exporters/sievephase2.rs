//! Export functionality for SIEVE IRs.

use std::io::{Error, ErrorKind, Result, Write};

use crate::exporters::Export;
use crate::Operation;

pub struct IR0;

impl Export<bool> for IR0 {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        match gate {
            Operation::Input(i) => {
                //NOTE(lisaoverall): needs to be updated for field switching
                writeln!(sink, "${} <- @private();", i)
            }
            Operation::Random(_) => Err(Error::new(
                ErrorKind::Other,
                "can't use random gates in IR1",
            )),
            Operation::Add(o, l, r) => {
                writeln!(sink, "${} <- @add(${}, ${});", o, l, r)
            }
            Operation::AddConst(o, i, c) => {
                writeln!(sink, "${} <- @addc(${}, < {} >);", o, i, *c as u32)
            }
            Operation::Sub(o, l, r) => {
                writeln!(sink, "${} <- @add(${}, ${});", o, l, r)
            }
            Operation::SubConst(o, i, c) => {
                writeln!(sink, "${} <- @addc(${}, < {} >);", o, i, *c as u32)
            }
            Operation::Mul(o, l, r) => {
                writeln!(sink, "${} <- @mul(${}, ${});", o, l, r)
            }
            Operation::MulConst(o, i, c) => {
                writeln!(sink, "${} <- @mulc(${}, < {} >);", o, i, *c as u32)
            }
            Operation::AssertZero(w) => {
                writeln!(sink, "@assert_zero(${});", w)
            }
            Operation::Const(w, c) => {
                writeln!(sink, "${} <- < {} >;", w, *c as u32)
            }
        }
    }

    fn export_circuit(gates: &[Operation<bool>], _: &[bool], sink: &mut impl Write) -> Result<()> {
        // Header fields.
        writeln!(sink, "version 2.0.0-beta;")?;
        writeln!(sink, "circuit;")?;
        writeln!(sink, "@type field 2;")?;

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

impl IR0 {
    fn export_input(
        witness: Option<&[bool]>,
        input_type: &str,
        sink: &mut impl Write,
    ) -> Result<()> {
        // Header fields.
        writeln!(sink, "version 2.0.0-beta;")?;
        writeln!(sink, "{};", input_type)?;
        writeln!(sink, "@type field 2;")?;

        // Private input body.
        writeln!(sink, "@begin")?;
        match witness {
            Some(w) => {
                for wit_value in w.iter() {
                    writeln!(sink, "< {} > ;", *wit_value as u32)?;
                }
            }
            None => (),
        };

        writeln!(sink, "@end")?;
        Ok(())
    }

    pub fn export_private_input(witness: &[bool], sink: &mut impl Write) -> Result<()> {
        IR0::export_input(Some(witness), "private_input", sink)
    }

    pub fn export_public_input(instance: Option<&[bool]>, sink: &mut impl Write) -> Result<()> {
        IR0::export_input(instance, "public_input", sink)
    }
}

#[cfg(test)]
mod tests {
    use crate::exporters::sievephase2::IR0;
    use crate::exporters::Export;
    use crate::Operation;

    #[test]
    fn print_example_circuit() {
        let mut sink = Vec::new();

        assert!(IR0::export_circuit(
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
            "version 2.0.0-beta;
circuit;
@type field 2;
@begin
$1 <- @private();
$2 <- @private();
$3 <- @private();
$4 <- @add($1, $3);
$5 <- @add($2, $3);
$6 <- @mul($5, $4);
$0 <- @addc($6, < 1 >);
@assert_zero($0);
@end
"
        );
    }

    #[test]
    fn print_example_private_input() {
        let mut sink = Vec::new();

        assert!(IR0::export_private_input(&[false, false, true], &mut sink,).is_ok());

        let bf = std::str::from_utf8(&sink).unwrap();
        assert_eq!(
            bf,
            "version 2.0.0-beta;
private_input;
@type field 2;
@begin
< 0 > ;
< 0 > ;
< 1 > ;
@end
"
        );
    }
}
