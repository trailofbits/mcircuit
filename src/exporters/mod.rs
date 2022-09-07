use std::io::{Result, Write};

use crate::{Operation, WireValue};

mod bristol;
mod json;

pub use bristol::bool_circuit_to_bristol;
pub use json::bool_circuit_to_json;

/// The core export trait.
///
/// Individual exporters (such as for Bristol-fashion circuits) are expected
/// to implement this trait.
pub trait Export<T: WireValue> {
    fn export_gate(gate: &Operation<T>, sink: &mut impl Write) -> Result<()>;

    fn export_circuit(gates: &[Operation<T>], witness: &[T], sink: &mut impl Write) -> Result<()>;
}
