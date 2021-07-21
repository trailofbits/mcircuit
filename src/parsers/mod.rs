use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

use crate::{Operation, WireValue};

mod smtlib;
mod json;
mod blif;

trait Parse<T: WireValue> {
    fn new(reader: BufReader<File>) -> Self;

    fn next(&mut self) -> Option<Operation<T>>;
}

struct WireHasher {
    hashes: HashMap<usize, usize>,
}

impl WireHasher {
    fn new() -> Self {
        WireHasher{
            hashes: HashMap::new(),
        }
    }

    fn get_wire_id(&mut self, name: &str) -> usize {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        let len = self.hashes.len();

        *self.hashes.entry(s.finish() as usize).or_insert(len)
    }
}

impl Default for WireHasher{
    fn default() -> Self {
        WireHasher::new()
    }
}