use crate::Operation;

fn bool_gate_to_bristol(gate: &Operation<bool>) -> String {
    match gate {
        Operation::Input(w) => {
            format!("0 1 {} INPUT", w)
        }
        Operation::Random(_) => {
            unimplemented!("Can't use random gates in Bristol")
        }
        Operation::Add(o, l, r) => {
            format!("2 1 {} {} {} XOR", l, r, o)
        }
        Operation::AddConst(o, i, c) => {
            if *c {
                format!("1 1 {} {} INV", i, o)
            } else {
                format!("1 1 {} {} EQW", i, o) // identity gate
            }
        }
        Operation::Sub(o, l, r) => {
            format!("2 1 {} {} {} XOR", l, r, o) // ADD and SUB are equivalent on GF2
        }
        Operation::SubConst(o, i, c) => {
            if *c {
                format!("1 1 {} {} INV", i, o)
            } else {
                format!("1 1 {} {} EQW", i, o) // identity gate
            }
        }
        Operation::Mul(o, l, r) => {
            format!("2 1 {} {} {} AND", l, r, o)
        }
        Operation::MulConst(o, i, c) => {
            if *c {
                format!("1 1 {} {} EQW", i, o) // identity gate
            } else {
                format!("1 1 0 {} EQ", o)
            }
        }
        Operation::AssertZero(w) => {
            // Bristol doesn't really have a concept of output wires _or_ assertions, so this
            // non-spec representation is the best we can do.
            format!("0 1 {} OUTPUT", w)
        }
        Operation::Const(w, c) => {
            format!("1 1 {} {} EQ", if *c { 1 } else { 0 }, w)
        }
    }
}

pub fn bool_circuit_to_bristol(gates: &[Operation<bool>], bool_witness: &[bool]) -> String {
    let mut circuit: String = String::new();
    let mut bool_iter = bool_witness.iter();

    for gate in gates {
        circuit.push_str(
            format!(
                "{}\n",
                match gate {
                    Operation::Input(o) => {
                        bool_gate_to_bristol(&Operation::Const(*o, *bool_iter.next().unwrap()))
                    }
                    _ => {
                        bool_gate_to_bristol(gate)
                    }
                }
            )
            .as_str(),
        )
    }

    circuit
}

#[cfg(test)]
mod tests {
    use crate::exporters::bristol::bool_circuit_to_bristol;
    use crate::Operation;

    #[test]
    fn print_example() {
        println!(
            "{}",
            bool_circuit_to_bristol(
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
                &[false, false, true]
            )
        );
    }
}
