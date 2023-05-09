use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::parsers::Parse;
use crate::WireValue;

pub struct WitnessParser<R: Read, const L: usize> {
    buf: String,
    reader: BufReader<R>,
}

impl<R: Read, const L: usize> Parse<bool, R> for WitnessParser<R, L> {
    type Item = [bool; L];

    fn new(reader: BufReader<R>) -> Self {
        WitnessParser {
            buf: String::with_capacity(L),
            reader: reader,
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.reader
            .read_line(&mut self.buf)
            .expect("failed to read from witness!");
        let wit: [bool; L] = self
            .buf
            .chars()
            .map(|c| match c {
                '0' => false,
                '1' => true,
                _ => panic!("bad bit {:?} in witness!", c),
            })
            .collect::<Vec<bool>>()
            .try_into()
            .expect("invalid trace step length!");
        self.buf.clear();
        Some(wit)
    }
}

impl<R: Read, const L: usize> Iterator for WitnessParser<R, L> {
    type Item = [bool; L];

    fn next(&mut self) -> Option<Self::Item> {
        Parse::next(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::witness::*;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_witness_parser() {
        let mut sink = BufReader::new("FUCK".as_bytes());
        let wp = WitnessParser::new(sink);

        let foo = wp.into_iter().collect();
    }
}
