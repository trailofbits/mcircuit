use std::io::{Result, Write};

use crate::{Operation, WireValue};

mod bristol;
mod json;
mod sieve;

pub use bristol::BristolFashion;
pub use json::bool_circuit_to_json;
pub use sieve::IR1;

/// The core export trait.
///
/// Individual exporters (such as for Bristol-fashion circuits) are expected
/// to implement this trait.
pub trait Export<T: WireValue> {
    fn export_gate(gate: &Operation<T>, sink: &mut impl Write) -> Result<()>;

    fn export_circuit(gates: &[Operation<T>], witness: &[T], sink: &mut impl Write) -> Result<()>;
}
