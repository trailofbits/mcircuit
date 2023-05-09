use std::io::Result;

use crate::{Witness, Operation, WireValue};

// NOTE(jl): 2023/05/09 fully deprecating Bristol, JSON, and IR1 backends.
// mod bristol;
// mod sieve;
// mod json;

mod sievephase2;

pub use sievephase2::IR0;

/// The core export trait.
///
/// Individual exporters (such as for Bristol-fashion circuits) are expected
/// to implement this trait.
pub trait Export<T: WireValue, const WITNESS_LEN: usize> {
    fn export(
        gates: &[Operation<T>],
        witness: Option<&Witness<WITNESS_LEN>>,
        out: &str,
    ) -> Result<()>;
}
