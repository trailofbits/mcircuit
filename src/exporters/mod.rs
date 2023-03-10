use std::io::{Result, Write};

use crate::{Operation, WireValue};

mod bristol;
mod sieve;
mod sievephase2;

pub use bristol::BristolFashion;
pub use sieve::IR1;
pub use sievephase2::IR0;

const WITNESS_LEN: usize = 656;

type Step = [bool; WITNESS_LEN];
type Witness = Vec<Step>;

/// The core export trait.
///
/// Individual exporters (such as for Bristol-fashion circuits) are expected
/// to implement this trait.
pub trait Export<T: WireValue> {
    fn export_gate(gate: &Operation<T>, sink: &mut impl Write) -> Result<()>;

    fn export_circuit(
        gates: &[Operation<T>],
        witness: &Witness,
        sink: &mut impl Write,
    ) -> Result<()>;
}
