use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

use crate::WireValue;

pub mod blif;
mod smtlib;

pub trait Parse<T: WireValue> {
    type Item;

    fn new(reader: BufReader<File>) -> Self;

    fn next(&mut self) -> Option<Self::Item>;
}

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

    pub fn backref(&self, id: usize) -> Option<&String> {
        None
    }
}

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
        if self.hashes.contains_key(&hash) {
            return *self.hashes.get(&hash).unwrap();
        } else {
            self.hashes.insert(hash, len);
            self.reverse.push(name.to_string());
            assert_eq!(self.reverse.len(), len + 1);
            len
        }
    }

    pub fn backref(&self, id: usize) -> Option<&String> {
        self.reverse.get(id)
    }
}

impl Default for WireHasher {
    fn default() -> Self {
        WireHasher::new()
    }
}
