//! Export functionality for SIEVE IRs.

use std::io::{Result, Write};

use crate::exporters::Export;
use crate::Operation;

pub struct IR1;

impl Export<bool> for IR1 {
    fn export_gate(gate: &Operation<bool>, sink: &mut impl Write) -> Result<()> {
        unimplemented!();
    }

    fn export_circuit(
        gates: &[Operation<bool>],
        witness: &[bool],
        sink: &mut impl Write,
    ) -> Result<()> {
        unimplemented!();
    }
}
