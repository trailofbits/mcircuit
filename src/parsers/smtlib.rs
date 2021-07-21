use std::io::BufReader;
use std::fs::File;
use crate::parsers::{WireHasher, Parse};
use crate::Operation;

struct SMTLibParser{
    reader: BufReader<File>,
    hasher: WireHasher
}

impl Parse<bool> for SMTLibParser{
    fn new(reader: BufReader<File>) -> Self {
        SMTLibParser{
            reader,
            hasher: Default::default(),
        }
    }

    fn next(&mut self) -> Option<Operation<bool>> {
        todo!()
    }
}