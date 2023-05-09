//! Export functionality for SIEVE IRs.

use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};

use crate::exporters::{Export, Witness};
use crate::Operation;

pub struct IR0;

impl<const WITNESS_LEN: usize> Export<bool, WITNESS_LEN> for IR0 {
    fn export(
        gates: &[Operation<bool>],
        witness: Option<&Witness<WITNESS_LEN>>,
        sink: &str,
    ) -> Result<()> {
        let witness = witness
            .ok_or_else(|| Error::new(ErrorKind::Other, "Witness is required for IR0 backend!"))?;

        // Header fields.
        let mut public = BufWriter::new(File::create(format!("{}.public_input", sink))?);
        let mut private = BufWriter::new(File::create(format!("{}.private_input", sink))?);
        let mut circuit = BufWriter::new(File::create(format!("{}.circuit", sink))?);

        IR0::export_circuit(gates, &mut circuit)?;
        IR0::export_private_input(witness, &mut private)?;
        IR0::export_public_input(None::<&Witness<WITNESS_LEN>>, &mut public)?;

        Ok(())
    }
}

impl IR0 {
    fn export_circuit(gates: &[Operation<bool>], sink: &mut impl Write) -> Result<()> {
        writeln!(sink, "version 2.0.0-beta;")?;
        writeln!(sink, "circuit;")?;
        writeln!(sink, "@type field 2;")?;

        // Circuit body.
        // We're allowed to emit functions in here, before any literal
        // gate directives. But we currently don't need that.
        writeln!(sink, "@begin")?;
        for gate in gates.iter() {
            writeln!(sink, "{}", IR0::export_gate(gate)?)?;
        }
        writeln!(sink, "@end")?;
        Ok(())
    }

    fn export_gate(gate: &Operation<bool>) -> Result<String> {
        match gate {
            Operation::Input(i) => {
                //NOTE(lisaoverall): needs to be updated for field switching
                Ok(format!("${} <- @private();", i))
            }
            Operation::Random(_) => Err(Error::new(
                ErrorKind::Other,
                "can't use random gates in IR1",
            )),
            Operation::Add(o, l, r) => Ok(format!("${} <- @add(${}, ${});", o, l, r)),
            Operation::AddConst(o, i, c) => {
                Ok(format!("${} <- @addc(${}, < {} >);", o, i, *c as u32))
            }
            Operation::Sub(o, l, r) => Ok(format!("${} <- @add(${}, ${});", o, l, r)),
            Operation::SubConst(o, i, c) => {
                Ok(format!("${} <- @addc(${}, < {} >);", o, i, *c as u32))
            }
            Operation::Mul(o, l, r) => Ok(format!("${} <- @mul(${}, ${});", o, l, r)),
            Operation::MulConst(o, i, c) => {
                Ok(format!("${} <- @mulc(${}, < {} >);", o, i, *c as u32))
            }
            Operation::AssertZero(w) => Ok(format!("@assert_zero(${});", w)),
            Operation::Const(w, c) => Ok(format!("${} <- < {} >;", w, *c as u32)),
        }
    }

    fn export_input<const L: usize>(
        witness: Option<&Witness<L>>,
        input_type: &str,
        sink: &mut impl Write,
    ) -> Result<()> {
        // Header fields.
        writeln!(sink, "version 2.0.0-beta;")?;
        writeln!(sink, "{};", input_type)?;
        writeln!(sink, "@type field 2;")?;

        // Input body.
        writeln!(sink, "@begin")?;
        if let Some(w) = witness {
            for wit_value in w.witness.iter().flatten() {
                writeln!(sink, "< {} > ;", *wit_value as u32)?;
            }
        }

        writeln!(sink, "@end")?;
        Ok(())
    }

    pub fn export_private_input<const L: usize>(
        witness: &Witness<L>,
        sink: &mut impl Write,
    ) -> Result<()> {
        IR0::export_input(Some(witness), "private_input", sink)
    }

    pub fn export_public_input<const L: usize>(
        instance: Option<&Witness<L>>,
        sink: &mut impl Write,
    ) -> Result<()> {
        IR0::export_input(instance, "public_input", sink)
    }
}

#[cfg(test)]
mod tests {
    use crate::exporters::sievephase2::IR0;
    use crate::exporters::{Export, Witness};
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

        assert!(IR0::export_private_input::<3>(
            &Witness {
                witness: vec![[false, false, true], [true, false, true]]
            },
            &mut sink
        )
        .is_ok());

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
< 1 > ;
< 0 > ;
< 1 > ;
@end
"
        );
    }
}
