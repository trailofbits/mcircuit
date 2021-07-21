use std::io::BufReader;
use std::fs::File;
use crate::parsers::{WireHasher, Parse};
use crate::Operation;

struct BlifParser{
    reader: BufReader<File>,
    hasher: WireHasher
}

impl Parse<bool> for BlifParser {
    fn new(reader: BufReader<File>) -> Self {
        BlifParser{
            reader,
            hasher: Default::default(),
        }
    }

    fn next(&mut self) -> Option<Operation<bool>> {
        todo!()
    }
}

impl Parse<u64> for BlifParser {
    fn new(reader: BufReader<File>) -> Self {
        BlifParser{
            reader,
            hasher: Default::default(),
        }
    }

    fn next(&mut self) -> Option<Operation<u64>> {
        todo!()
    }
}