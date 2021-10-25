use serde_json::{Result, Value};

use crate::{Operation, WireValue};

fn gate_to_json<T: WireValue>(gate: &Operation<T>) -> Value {
    unimplemented!("JSON exporter is private for now");
}

pub fn bool_circuit_to_json(gates: &[Operation<bool>], bool_witness: &[bool]) -> Result<String> {
    unimplemented!("JSON exporter is private for now");
}
