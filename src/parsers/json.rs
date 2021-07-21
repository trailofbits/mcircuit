use std::array::IntoIter;
use std::iter::FromIterator;

use serde_json::{Map, Number, Result, Value};

use crate::{Operation, WireValue};

/*
IR1 mask flags - copied and pasted, so they may get out of sync
 */

// Arithmetic Gates
pub const ADD: u16 = 0x0001;
pub const ADDC: u16 = 0x0002;
pub const MUL: u16 = 0x0004;
pub const MULC: u16 = 0x0008;
pub const ARITH: u16 = ADD | ADDC | MUL | MULC;

// Boolean Gates
pub const XOR: u16 = 0x0100;
pub const AND: u16 = 0x0200;
pub const NOT: u16 = 0x0400;
pub const BOOL: u16 = XOR | AND | NOT;

// Various features
pub const FUNCTION: u16 = 0x1000;
pub const FOR: u16 = 0x2000;
pub const SWITCH: u16 = 0x4000;
pub const FOR_FUNCTION_SWITCH: u16 = FOR | FUNCTION | SWITCH;
pub const SIMPLE: u16 = 0x0000;

// Our own flags
/// Characteristic of the Galois field (2 for boolean circuits)
const FIELD_CHAR: u64 = 2;
/// Field degree, should probably be 1 unless you know what you're doing
const FIELD_DEG: u64 = 1;
/// Tell ZK_Interface which gates we allow
const GATE_MASK: u16 = ARITH;
/// Tell ZK_Interface which features this circuit will use (none?)
const FEAT_MASK: u16 = SIMPLE;

fn make_header(field_char: u64, field_deg: u64) -> Value {
    Value::Object(Map::from_iter(IntoIter::new([
        ("version".to_owned(), Value::String("1.0.0".to_owned())),
        (
            "field_characteristic".to_owned(),
            Value::Array(vec![Value::Number(Number::from(field_char))]),
        ),
        (
            "field_degree".to_owned(),
            Value::Number(Number::from(field_deg)),
        ),
    ])))
}

fn gate_to_json<T: WireValue>(gate: &Operation<T>) -> Value {
    // TODO: For identity gates, emit Copy
    Value::Object(match *gate {
        Operation::Input(o) => Map::from_iter(IntoIter::new([(
            "Witness".to_owned(),
            Value::Number(Number::from(o)),
        )])),
        Operation::Add(o, a, b) => Map::from_iter(IntoIter::new([(
            "Add".to_owned(),
            Value::Array(vec![
                Value::Number(Number::from(o)),
                Value::Number(Number::from(a)),
                Value::Number(Number::from(b)),
            ]),
        )])),
        Operation::AddConst(o, i, val) => Map::from_iter(IntoIter::new([(
            "AddConstant".to_owned(),
            Value::Array(vec![
                Value::Number(Number::from(o)),
                Value::Number(Number::from(i)),
                Value::Array(
                    val.to_le_bytes()
                        .iter()
                        .map(|b| Value::Number(Number::from(*b)))
                        .collect(),
                ),
            ]),
        )])),
        Operation::Mul(o, a, b) => Map::from_iter(IntoIter::new([(
            "Mul".to_owned(),
            Value::Array(vec![
                Value::Number(Number::from(o)),
                Value::Number(Number::from(a)),
                Value::Number(Number::from(b)),
            ]),
        )])),
        Operation::MulConst(o, i, val) => Map::from_iter(IntoIter::new([(
            "MulConstant".to_owned(),
            Value::Array(vec![
                Value::Number(Number::from(o)),
                Value::Number(Number::from(i)),
                Value::Array(
                    val.to_le_bytes()
                        .iter()
                        .map(|b| Value::Number(Number::from(*b)))
                        .collect(),
                ),
            ]),
        )])),
        Operation::AssertZero(i) => Map::from_iter(IntoIter::new([(
            "AssertZero".to_owned(),
            Value::Number(Number::from(i)),
        )])),
        Operation::Const(o, val) => Map::from_iter(IntoIter::new([(
            "Constant".to_owned(),
            Value::Array(vec![
                Value::Number(Number::from(o)),
                Value::Array(
                    val.to_le_bytes()
                        .iter()
                        .map(|b| Value::Number(Number::from(*b)))
                        .collect(),
                ),
            ]),
        )])),
        Operation::Sub(_, _, _) => {
            unimplemented!(
                "Subtraction gates are not supported by IR1; \
            you should convert them into equivalent Addition gates on this field."
            )
        }
        Operation::SubConst(_, _, _) => {
            unimplemented!(
                "Subtraction gates are not supported by IR1; \
            you should convert them into equivalent Addition gates on this field."
            )
        }
        // Operation::Random(_) => {}
        _ => unimplemented!("Can't export {:?} in IR1 yet!", gate),
    })
}

fn bool_circuit_to_json(gates: &[Operation<bool>], bool_witness: &[bool]) -> Result<String> {
    let ir_rep = Value::Object(Map::from_iter(IntoIter::new([
        (
            "instances".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter(IntoIter::new([
                ("header".to_owned(), make_header(FIELD_CHAR, FIELD_DEG)),
                ("common_inputs".to_owned(), Value::Array(vec![])),
            ])))]),
        ),
        (
            "witnesses".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter(IntoIter::new([
                ("header".to_owned(), make_header(FIELD_CHAR, FIELD_DEG)),
                (
                    "short_witness".to_owned(),
                    Value::Array(
                        bool_witness
                            .iter()
                            .map(|b| {
                                Value::Array(
                                    b.to_le_bytes()
                                        .iter()
                                        .map(|u| Value::Number(Number::from(*u)))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ),
                ),
            ])))]),
        ),
        (
            "relations".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter(IntoIter::new([
                ("header".to_owned(), make_header(FIELD_CHAR, FIELD_DEG)),
                (
                    "gate_mask".to_owned(),
                    Value::Number(Number::from(GATE_MASK)),
                ),
                (
                    "feat_mask".to_owned(),
                    Value::Number(Number::from(FEAT_MASK)),
                ),
                (
                    "functions".to_owned(),
                    Value::Array(vec![
                        // Function objects would go here
                    ]),
                ),
                (
                    "gates".to_owned(),
                    Value::Array(gates.iter().map(|g| gate_to_json(g)).collect()),
                ),
            ])))]),
        ),
    ])));

    serde_json::to_string(&ir_rep)
}

#[cfg(test)]
mod tests {
    use crate::parsers::json::bool_circuit_to_json;
    use crate::Operation;

    #[test]
    fn print_example() {
        println!(
            "{}",
            bool_circuit_to_json(
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
            .unwrap()
        );
    }
}
