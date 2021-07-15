use std::array::IntoIter;
use std::iter::FromIterator;

use serde_json::{Map, Number, Result, Value};

use crate::{Operation, WireValue};

const FIELD_CHAR: i32 = 101;
const FIELD_DEG: i32 = 1;
const GATE_MASK: i32 = 5;
const FEAT_MASK: i32 = 20480;

fn make_header(field_char: i32, field_deg: i32) -> Value {
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

fn arith_circuit_to_json(gates: &[Operation<u64>]) -> Result<String> {
    let ir_rep = Value::Object(Map::from_iter(IntoIter::new([
        (
            "instances".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter(IntoIter::new([
                ("header".to_owned(), make_header(FIELD_CHAR, FIELD_DEG)),
                (
                    "common_inputs".to_owned(),
                    Value::Array(vec![Value::Array(vec![
                        // Value::Number would go here
                    ])]),
                ),
            ])))]),
        ),
        (
            "witnesses".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter(IntoIter::new([
                ("header".to_owned(), make_header(FIELD_CHAR, FIELD_DEG)),
                (
                    "short_witness".to_owned(),
                    Value::Array(vec![Value::Array(vec![
                        // Value::Number would go here
                    ])]),
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
    use crate::export_json::arith_circuit_to_json;

    #[test]
    fn print_example() {
        println!("{}", arith_circuit_to_json(&[]).unwrap());
    }
}
