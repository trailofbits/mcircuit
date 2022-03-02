use serde_json::{Result, Value};

use crate::{Operation, WireValue};

fn _gate_to_json<T: WireValue>(_gate: &Operation<T>) -> Value {
    unimplemented!("JSON exporter is private for now");
}

pub fn bool_circuit_to_json(_gates: &[Operation<bool>], _bool_witness: &[bool]) -> Result<String> {
    unimplemented!("JSON exporter is private for now");
}
