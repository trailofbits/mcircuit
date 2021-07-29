use std::collections::VecDeque;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem::swap;
use std::mem::take;

use crate::parsers::{Parse, WireHasher};
use crate::WireValue;
use crate::{OpType, Operation};
use num_traits::Zero;

pub fn parse_split(pair: &str) -> (&str, &str) {
    let mut split = pair.split('=');
    (split.next().unwrap(), split.next().unwrap())
}

fn parse_gate(mut line: VecDeque<&str>) -> (&str, &str, Vec<&str>) {
    let op = line.pop_front().unwrap();
    let (_, out) = parse_split(line.pop_back().unwrap());
    let inputs: Vec<&str> = line.drain(..).map(|part| parse_split(part).1).collect();

    (op, out, inputs)
}

fn parse_io(mut line: VecDeque<&str>) -> Vec<Vec<&str>> {
    let mut out: Vec<Vec<&str>> = Vec::new();
    while !line.is_empty() {
        let mut chunk: Vec<&str> = Vec::new();
        let start_token = line[0].split('[').next().unwrap();
        while !line.is_empty() && line[0].starts_with(start_token) {
            chunk.push(line.pop_front().unwrap());
        }
        let to_push = take(&mut chunk);
        out.push(to_push);
    }
    out
}

fn parse_subcircuit(mut line: VecDeque<&str>) -> (&str, Vec<(&str, &str)>) {
    let name = line.pop_front().unwrap();
    let io: Vec<(&str, &str)> = line.drain(..).map(|part| parse_split(part)).collect();

    (name, io)
}

#[derive(Clone)]
pub struct BlifCircuitDesc<T: WireValue> {
    pub name: String,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub gates: Vec<Operation<T>>,
    pub subcircuits: Vec<BlifSubcircuitDesc>,
}

#[derive(Clone)]
pub struct BlifSubcircuitDesc {
    pub name: String,
    pub connections: Vec<(usize, usize)>,
}

impl Default for BlifSubcircuitDesc {
    fn default() -> Self {
        BlifSubcircuitDesc {
            name: "".to_string(),
            connections: vec![],
        }
    }
}

impl<T: WireValue> Default for BlifCircuitDesc<T> {
    fn default() -> Self {
        BlifCircuitDesc {
            name: "".to_string(),
            inputs: vec![],
            outputs: vec![],
            gates: vec![],
            subcircuits: vec![],
        }
    }
}

pub trait CanConstructVariant<T: WireValue> {
    fn construct_variant(
        &mut self,
        op: &str,
        out: usize,
        inputs: &[usize],
        cons: Option<T>,
    ) -> Operation<T>;
}

pub struct BlifParser<T: WireValue> {
    reader: Option<BufReader<File>>,
    pub hasher: WireHasher,
    parsed: bool,
    circuit: Vec<BlifCircuitDesc<T>>,
}

impl<T: WireValue> Default for BlifParser<T> {
    fn default() -> Self {
        BlifParser {
            reader: None,
            hasher: Default::default(),
            parsed: false,
            circuit: vec![],
        }
    }
}

impl CanConstructVariant<bool> for BlifParser<bool> {
    fn construct_variant(
        &mut self,
        op: &str,
        out: usize,
        inputs: &[usize],
        cons: Option<bool>,
    ) -> Operation<bool> {
        match op {
            "AND" | "MUL" => Operation::construct(
                OpType::Binary(Operation::Mul),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "XOR" | "ADD" => Operation::construct(
                OpType::Binary(Operation::Add),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "NOT" | "INV" => Operation::construct(
                OpType::BinaryConst(Operation::AddConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                Some(true),
            ),
            "BUF" => Operation::construct(
                OpType::BinaryConst(Operation::AddConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                Some(false),
            ),
            "RAND" => Operation::construct(
                OpType::Input(Operation::Random),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "CONST" => Operation::construct(
                OpType::InputConst(Operation::Const),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                cons,
            ),
            _ => unimplemented!("Unsupported gate type: {}", op),
        }
    }
}

impl CanConstructVariant<u64> for BlifParser<u64> {
    fn construct_variant(
        &mut self,
        op: &str,
        out: usize,
        inputs: &[usize],
        cons: Option<u64>,
    ) -> Operation<u64> {
        match op {
            "MUL" => Operation::construct(
                OpType::Binary(Operation::Mul),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "MULC" => Operation::construct(
                OpType::BinaryConst(Operation::MulConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                cons,
            ),
            "ADD" => Operation::construct(
                OpType::Binary(Operation::Add),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "ADDC" => Operation::construct(
                OpType::BinaryConst(Operation::AddConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                cons,
            ),
            "SUB" => Operation::construct(
                OpType::Binary(Operation::Sub),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "SUBC" => Operation::construct(
                OpType::BinaryConst(Operation::SubConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                cons,
            ),
            "BUF" => Operation::construct(
                OpType::BinaryConst(Operation::AddConst),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                Some(u64::zero()),
            ),
            "RAND" => Operation::construct(
                OpType::Input(Operation::Random),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                None,
            ),
            "CONST" => Operation::construct(
                OpType::InputConst(Operation::Const),
                inputs.iter().copied(),
                vec![out].iter().copied(),
                cons,
            ),
            _ => unimplemented!("Unsupported gate type: {}", op),
        }
    }
}

impl<T: WireValue> BlifParser<T>
where
    BlifParser<T>: CanConstructVariant<T>,
{
    fn clean_parse(&mut self) {
        self.parsed = true;

        if self.reader.is_some() {
            let mut reader: Option<BufReader<File>> = None;
            swap(&mut reader, &mut self.reader);

            // reserve the 0 and 1 wires for true and false
            assert_eq!(self.hasher.get_wire_id("$false"), 0);
            assert_eq!(self.hasher.get_wire_id("$true"), 1);

            let mut current: BlifCircuitDesc<T> = Default::default();

            for line in reader.unwrap().lines().flatten() {
                let mut line: VecDeque<&str> = line.trim().split(' ').collect();
                let cmd = line.pop_front().unwrap();
                match cmd {
                    ".model" => {
                        current.name = line.pop_front().unwrap().into();
                    }
                    ".inputs" => {
                        for chunk in parse_io(line) {
                            for name in chunk.iter().rev() {
                                let id = self.hasher.get_wire_id(name);
                                current.inputs.push(id);
                            }
                        }
                    }
                    ".outputs" => {
                        for chunk in parse_io(line) {
                            for name in chunk.iter().rev() {
                                let id = self.hasher.get_wire_id(name);
                                current.outputs.push(id);
                            }
                        }
                    }
                    ".gate" => {
                        let (op, out, mut inputs) = parse_gate(line);
                        let out_id = self.hasher.get_wire_id(out);
                        let input_ids: Vec<usize> = inputs
                            .drain(..)
                            .map(|name| self.hasher.get_wire_id(name))
                            .collect();
                        current
                            .gates
                            .push(self.construct_variant(op, out_id, &input_ids, None));
                    }
                    ".subckt" => {
                        let (name, mut io_pairings) = parse_subcircuit(line);
                        let connections = io_pairings
                            .drain(..)
                            .map(|(child_name, parent_name)| {
                                (
                                    self.hasher.get_wire_id(parent_name),
                                    self.hasher.get_wire_id(child_name),
                                )
                            })
                            .collect();

                        current.subcircuits.push(BlifSubcircuitDesc {
                            name: name.into(),
                            connections,
                        })
                    }
                    ".names" | ".conn" => {
                        let from = self.hasher.get_wire_id(line.pop_front().unwrap());
                        let to = self.hasher.get_wire_id(line.pop_back().unwrap());
                        current
                            .gates
                            .push(self.construct_variant("BUF", to, &[from], None))
                    }
                    ".end" => {
                        self.circuit.push(take(&mut current));
                    }
                    _ => (),
                }
            }
            self.circuit.reverse();
        }
    }
}

impl<T: WireValue> Parse<T> for BlifParser<T>
where
    BlifParser<T>: CanConstructVariant<T>,
{
    type Item = BlifCircuitDesc<T>;

    fn new(reader: BufReader<File>) -> Self {
        BlifParser {
            reader: Some(reader),
            ..Default::default()
        }
    }

    fn next(&mut self) -> Option<BlifCircuitDesc<T>> {
        if !self.parsed {
            self.clean_parse();
        }
        self.circuit.pop()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::parsers::blif::{parse_gate, parse_io, parse_subcircuit};

    #[test]
    fn test_gate_parsing() {
        let line: VecDeque<&str> = "AND A=InputA B=InputB OUT=Output"
            .trim()
            .split(" ")
            .collect();
        let (op, out, inputs) = parse_gate(line);
        assert_eq!(op, "AND");
        assert_eq!(out, "Output");
        assert_eq!(inputs, vec!["InputA", "InputB"]);
    }

    #[test]
    fn test_io_parsing() {
        let line: VecDeque<&str> = "X[0] X[1] X[2] X[3] Y[0] Y[1] Y[2] Y[3] C_"
            .trim()
            .split(" ")
            .collect();
        let chunks = parse_io(line);
        assert_eq!(
            chunks,
            vec![
                vec!["X[0]", "X[1]", "X[2]", "X[3]"],
                vec!["Y[0]", "Y[1]", "Y[2]", "Y[3]"],
                vec!["C_"]
            ]
        );
    }

    #[test]
    fn test_subcircuit_parsing() {
        let line: VecDeque<&str> = "memTraceEntryEncoder address[0]=src_read_address[0] address[1]=src_read_address[1] address[2]=src_read_address[2]"
            .trim()
            .split(" ")
            .collect();
        let (op, pairings) = parse_subcircuit(line);
        assert_eq!(op, "memTraceEntryEncoder");
        assert_eq!(
            pairings,
            vec![
                ("address[0]", "src_read_address[0]"),
                ("address[1]", "src_read_address[1]"),
                ("address[2]", "src_read_address[2]"),
            ]
        );
    }
}
