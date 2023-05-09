use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Read};

/// TODO: WireHasher really ought to be a trait so that we can have a `Hasher` and `BackrefHasher`,
/// and not have to worry about hiding `backref` and the data that we need to back it up behind such
/// a complicated compile-time cfg.
use crate::WireValue;

pub mod blif;
pub mod witness;

pub trait Parse<T: WireValue, R: Read> {
    type Item;

    fn new(reader: BufReader<R>) -> Self;

    fn next(&mut self) -> Option<Self::Item>;
}

/// Calculates and remembers sequential hashes of wire names.
#[cfg(not(debug_assertions))]
pub struct WireHasher {
    hashes: HashMap<usize, usize>,
}

#[cfg(not(debug_assertions))]
impl WireHasher {
    fn new() -> Self {
        WireHasher {
            hashes: HashMap::new(),
        }
    }

    pub fn get_wire_id(&mut self, name: &str) -> usize {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        let len = self.hashes.len();

        *self.hashes.entry(s.finish() as usize).or_insert(len)
    }

    /// Allows you to map back to the string that created this hash. Only works in debug mode.
    pub fn backref(&self, id: usize) -> Option<&String> {
        None
    }
}

/// Calculates and remembers sequential hashes of wire names. For example:
/// ```
/// use mcircuit::parsers::WireHasher;
/// let mut hasher = WireHasher::default();
///
/// assert_eq!(hasher.get_wire_id("foo"), 0);
/// assert_eq!(hasher.get_wire_id("bar"), 1);
/// assert_eq!(hasher.get_wire_id("baz"), 2);
/// assert_eq!(hasher.get_wire_id("foo"), 0);
/// assert_eq!(hasher.get_wire_id("baz"), 2);
/// ```
#[cfg(debug_assertions)]
pub struct WireHasher {
    hashes: HashMap<usize, usize>,
    reverse: Vec<String>,
}

#[cfg(debug_assertions)]
impl WireHasher {
    fn new() -> Self {
        WireHasher {
            hashes: HashMap::new(),
            reverse: Vec::new(),
        }
    }

    pub fn get_wire_id(&mut self, name: &str) -> usize {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        let len = self.hashes.len();

        let hash = s.finish() as usize;
        match self.hashes.entry(hash) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                e.insert(len);
                self.reverse.push(name.to_string());
                assert_eq!(self.reverse.len(), len + 1);
                len
            }
        }
    }

    /// Allows you to map back to the string that created this hash. Only works in debug mode.
    pub fn backref(&self, id: usize) -> Option<&String> {
        self.reverse.get(id)
    }
}

impl Default for WireHasher {
    fn default() -> Self {
        WireHasher::new()
    }
}
