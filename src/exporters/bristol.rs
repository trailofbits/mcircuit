use std::collections::HashSet;
use std::io::{Error, ErrorKind, Result, Write};

use crate::exporters::{Export, WITNESS_LEN};
use crate::io_extractors::{InputIterator, OutputIterator};
use crate::Operation;

pub struct BristolFashion;

impl Export<bool> for BristolFashion {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        match gate {
            Operation::Input(w) => {
                writeln!(sink, "0 1 {} INPUT", w)
            }
            Operation::Random(_) => Err(Error::new(
                ErrorKind::Other,
                "can't use random gates in Bristol",
            )),
            Operation::Add(o, l, r) => {
                writeln!(sink, "2 1 {} {} {} XOR", l, r, o)
            }
            Operation::AddConst(o, i, c) => {
                if *c {
                    writeln!(sink, "1 1 {} {} INV", i, o)
                } else {
                    writeln!(sink, "1 1 {} {} EQW", i, o) // identity gate
                }
            }
            Operation::Sub(o, l, r) => {
                writeln!(sink, "2 1 {} {} {} XOR", l, r, o) // ADD and SUB are equivalent on GF2
            }
            Operation::SubConst(o, i, c) => {
                if *c {
                    writeln!(sink, "1 1 {} {} INV", i, o)
                } else {
                    writeln!(sink, "1 1 {} {} EQW", i, o) // identity gate
                }
            }
            Operation::Mul(o, l, r) => {
                writeln!(sink, "2 1 {} {} {} AND", l, r, o)
            }
            Operation::MulConst(o, i, c) => {
                if *c {
                    writeln!(sink, "1 1 {} {} EQW", i, o) // identity gate
                } else {
                    writeln!(sink, "1 1 0 {} EQ", o)
                }
            }
            Operation::AssertZero(w) => {
                // Bristol doesn't really have a concept of output wires _or_ assertions, so this
                // non-spec representation is the best we can do.
                writeln!(sink, "0 1 {} OUTPUT", w)
            }
            Operation::Const(w, c) => {
                writeln!(sink, "1 1 {} {} EQ", i32::from(*c), w)
            }
        }
    }

    fn export_circuit(
        gates: &[Operation<bool>],
        witness: &Vec<[bool; WITNESS_LEN]>,
        sink: &mut impl Write,
    ) -> Result<()> {
        // Every Bristol Fashion circuit begins with a "header", which predeclares
        // a few different input an output cardinalities. It looks like this:
        //
        //     {ngates} {nwires}
        //     {niv} {ni_1,...,ni_niv}
        //     {nov} {no_1,...,no_nov}
        //
        // Where {ngates} is the total number of gates, {nwires} is the total
        // number of wires, {niv} and {nov} are the number of input and output
        // values, respectively, and the lists that follow them describe the
        // number of wires per output value.
        //
        // For example, a circuit with 6 gates, 12 wires, 2 input values of
        // 1 wire each, and 1 output value of 1 wire would look like this:
        //
        //     6 12
        //     2 1 1
        //     1 1

        let mut wires = HashSet::new();
        let mut output_count = 0;
        for gate in gates {
            // Add all input and output wires in the operation to the set of seen wires.
            wires.extend(InputIterator::new(gate));
            wires.extend(OutputIterator::new(gate));

            if matches!(gate, Operation::AssertZero(_)) {
                output_count += 1;
            }
        }

        // {ngates} {nwires}
        writeln!(sink, "{} {}", gates.len(), wires.len())?;

        // {niv} {ni_1,...,ni_niv}
        // Each input is 1 bit.
        writeln!(
            sink,
            "{} {}",
            witness.len() * WITNESS_LEN,
            std::iter::repeat("1")
                .take(witness.len() * WITNESS_LEN)
                .collect::<Vec<_>>()
                .join(" ")
        )?;

        // {nov} {no_1,...,no_nov}
        // Each output is 1 bit...I think.
        writeln!(
            sink,
            "{} {}",
            output_count,
            std::iter::repeat("1")
                .take(output_count)
                .collect::<Vec<_>>()
                .join(" ")
        )?;

        let mut wit_iter = witness.iter().flatten();

        for gate in gates {
            match gate {
                Operation::Input(o) => Self::export_gate(
                    &Operation::Const(
                        *o,
                        *wit_iter
                            .next()
                            .ok_or_else(|| Error::new(ErrorKind::Other, "witness too short"))?,
                    ),
                    sink,
                )?,
                _ => Self::export_gate(gate, sink)?,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::exporters::bristol::BristolFashion;
    use crate::exporters::Export;
    use crate::Operation;

    #[test]
    fn print_example() {
        let mut sink = Vec::new();

        assert!(BristolFashion::export_circuit(
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
            "8 7\n3 1 1 1\n1 1\n1 1 0 1 EQ\n1 1 0 2 EQ\n1 1 1 3 EQ\n2 1 1 3 4 XOR\n2 1 2 3 5 XOR\n2 1 5 4 6 AND\n1 1 6 0 INV\n0 1 0 OUTPUT\n"
        );
    }
}
