use std::io::{Result, Write};

use crate::{Operation, WireValue};

mod bristol;
mod json;
mod sieve;
mod sievephase2;

pub use bristol::BristolFashion;
pub use json::bool_circuit_to_json;
pub use sieve::IR1;
pub use sievephase2::IR0;

const WITNESS_LEN: usize = 656;

/// The core export trait.
///
/// Individual exporters (such as for Bristol-fashion circuits) are expected
/// to implement this trait.
pub trait Export<T: WireValue> {
    fn export_gate(gate: &Operation<T>, sink: &mut impl Write) -> Result<()>;

    fn export_circuit(
        gates: &[Operation<T>],
        witness: &Vec<[bool; WITNESS_LEN]>,
        sink: &mut impl Write,
    ) -> Result<()>;
}
