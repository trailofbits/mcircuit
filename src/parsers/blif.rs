use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem::take;
use std::mem::{replace, swap};

use crate::parsers::{Parse, WireHasher};
use crate::{HasIO, WireValue};
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
    let io: Vec<(&str, &str)> = line.drain(..).map(parse_split).collect();

    (name, io)
}

fn get_base_name_and_width(unparsed: String) -> (String, usize) {
    let (base_name, after) = match unparsed.split_once('[') {
        None => {(unparsed, None)}
        Some((before, after)) => {(before.to_string(), Some(after))}
    };
    let idx = match after {
        None => {0}
        Some(after) => { usize::from_str_radix(after.split_once(']').unwrap().0, 10).unwrap() }
    };
    (base_name, idx)
}

#[derive(Clone)]
pub struct BlifCircuitDesc<T: WireValue> {
    pub name: String,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub gates: Vec<Operation<T>>,
    pub subcircuits: Vec<BlifSubcircuitDesc>,
}

impl<T: WireValue> BlifCircuitDesc<T> {
    fn add_subcircuit(&mut self, mut packed_sub: PackedSubcircuitDesc, hasher: &mut WireHasher) {
        let mut unpacked = BlifSubcircuitDesc {
            name: packed_sub.name.clone(),
            ..Default::default()
        };

        for (parent, sub) in packed_sub.connections.drain(..) {

            let (base_name_sub, packed_idx_sub) = get_base_name_and_width(sub.clone());
            let (base_name_parent, packed_idx_parent) = get_base_name_and_width(parent.clone());

            // Figure out if this wire is packed
            match packed_sub.packed_wires.get(base_name_sub.as_str()) {
                Some(width) => {

                    // Split wire into sub-wires
                    for i in 0..*width {
                        unpacked.connections.push((
                            hasher.get_wire_id(format!("{}[{}]", parent, i).as_str()),
                            hasher.get_wire_id(format!("{}[{}]", base_name_sub, (packed_idx_sub * width) + i).as_str()),
                        ));
                    }

                    // Check if any of the I/O wires are also packed
                    let parent_id = hasher.get_wire_id(&parent);

                    let mut new_inputs = Vec::with_capacity(self.inputs.len() + width);
                    for input in self.inputs.drain(..) {
                        if input == parent_id {
                            for i in 0..*width {
                                new_inputs
                                    .push(hasher.get_wire_id(format!("{}[{}]", parent, i).as_str()))
                            }
                        } else {
                            new_inputs.push(input)
                        }
                    }
                    self.inputs = new_inputs;

                    let mut new_outputs = Vec::with_capacity(self.outputs.len() + width);
                    for output in self.outputs.drain(..) {
                        if output == parent_id {
                            for i in 0..*width {
                                new_outputs
                                    .push(hasher.get_wire_id(format!("{}[{}]", parent, i).as_str()))
                            }
                        } else {
                            new_outputs.push(output)
                        }
                    }
                    self.outputs = new_outputs;

                    // Make sure we only connect to other subcircuits or I/O (not gates)
                    for gate in self.gates.iter() {
                        if gate.outputs().any(|w| w == parent_id) {
                            panic!(
                                "Tried to drive the packed wire {}.{} via an unpacked gate",
                                packed_sub.name, sub
                            );
                        }
                        if gate.inputs().any(|w| w == parent_id) {
                            panic!(
                                "Tried to drive an unpacked gate via packed wire {}.{}",
                                packed_sub.name, sub
                            );
                        }
                    }
                }
                // Otherwise this wire is fine, so just push it to the subcircuit
                None => {
                    unpacked
                        .connections
                        .push((hasher.get_wire_id(&parent), hasher.get_wire_id(&sub)));
                }
            }
        }

        self.subcircuits.push(unpacked);
    }
}

#[derive(Clone)]
pub struct BlifSubcircuitDesc {
    pub name: String,
    pub connections: Vec<(usize, usize)>,
}

#[derive(Clone)]
pub struct PackedSubcircuitDesc {
    pub name: String,
    pub connections: Vec<(String, String)>,
    pub packed_wires: HashMap<String, usize>,
}

impl Default for PackedSubcircuitDesc {
    fn default() -> Self {
        PackedSubcircuitDesc {
            name: "".to_string(),
            connections: vec![],
            packed_wires: HashMap::new(),
        }
    }
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

    fn constant_from_str(&self, s: &str) -> T;
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

    fn constant_from_str(&self, s: &str) -> bool {
        match s {
            "$false" => false,
            "$true" => true,
            _ => s
                .parse()
                .unwrap_or_else(|_| panic!("Can't convert {} into a bool", s)),
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

    fn constant_from_str(&self, s: &str) -> u64 {
        match s {
            "$false" => 0u64,
            "$true" => 1u64,
            _ => s
                .parse()
                .unwrap_or_else(|_| panic!("Can't convert {} into a u64", s)),
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

            let mut current: BlifCircuitDesc<T> = Default::default();
            let mut current_subcircuit: Option<PackedSubcircuitDesc> = None;

            // reserve the 0 and 1 wires for true and false.
            assert_eq!(self.hasher.get_wire_id("$false"), 0);
            assert_eq!(self.hasher.get_wire_id("$true"), 1);

            // Push const gates for true & false
            current.gates.push(self.construct_variant(
                "CONST",
                0,
                &[],
                Some(self.constant_from_str("$false")),
            ));
            current.gates.push(self.construct_variant(
                "CONST",
                1,
                &[],
                Some(self.constant_from_str("$true")),
            ));

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
                        if let Some(subc) =
                            replace(&mut current_subcircuit, Some(Default::default()))
                        {
                            // Push the previous subcircuit (if any)
                            current.add_subcircuit(subc, &mut self.hasher);
                        }

                        let (name, mut io_pairings) = parse_subcircuit(line);
                        let connections = io_pairings.drain(..).map(|(child_name, parent_name)| {
                            (parent_name.into(), child_name.into())
                        });

                        // Should _always_ be Some thanks to the earlier `replace`
                        if let Some(subc) = &mut current_subcircuit {
                            subc.name = name.into();
                            subc.connections.extend(connections);
                        }
                    }
                    ".attr" => {
                        let name = line.pop_front().unwrap();
                        let value = line.pop_front().unwrap();

                        match name {
                            "module_not_derived" => (),
                            "src" => (),
                            "_packing" => match value {
                                "\"gf2\"" => (),
                                _ => {
                                    unimplemented!("Unknown field: {}", value);
                                }
                            },
                            wire => {
                                let width = usize::from_str_radix(value, 2).unwrap();
                                if let Some(subc) = &mut current_subcircuit {
                                    subc.packed_wires.insert(wire.into(), width);
                                }
                            }
                        }
                    }
                    ".names" | ".conn" => {
                        let from = self.hasher.get_wire_id(line.pop_front().unwrap());
                        let to = self.hasher.get_wire_id(line.pop_back().unwrap());
                        current
                            .gates
                            .push(self.construct_variant("BUF", to, &[from], None))
                    }
                    ".end" => {
                        // Finish off any subcircuits that haven't been pushed
                        if let Some(subc) = take(&mut current_subcircuit) {
                            current.add_subcircuit(subc, &mut self.hasher);
                        }

                        if current.gates.len() == 0 {
                            println!("Warning: Dropping empty module {}", current.name);

                            current = Default::default();

                            current.gates.push(self.construct_variant(
                                "CONST",
                                0,
                                &[],
                                Some(self.constant_from_str("$false")),
                            ));
                            current.gates.push(self.construct_variant(
                                "CONST",
                                1,
                                &[],
                                Some(self.constant_from_str("$true")),
                            ));

                            continue;
                        }
                        else {
                            self.circuit.push(take(&mut current));
                            // Push const gates for true & false
                            current.gates.push(self.construct_variant(
                                "CONST",
                                0,
                                &[],
                                Some(self.constant_from_str("$false")),
                            ));
                            current.gates.push(self.construct_variant(
                                "CONST",
                                1,
                                &[],
                                Some(self.constant_from_str("$true")),
                            ));
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn add_file(&mut self, new_reader: BufReader<File>) {
        if !self.parsed {
            self.clean_parse();
        }

        self.reader = Some(new_reader);
        self.parsed = false;
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
        if !self.circuit.is_empty(){
            Some(self.circuit.remove(0))
        }
        else {
            None
        }

    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::parsers::blif::{get_base_name_and_width, parse_gate, parse_io, parse_subcircuit};

    #[test]
    fn test_gate_parsing() {
        let line: VecDeque<&str> = "AND A=InputA B=InputB OUT=Output"
            .trim()
            .split(' ')
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
            .split(' ')
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
            .split(' ')
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

    #[test]
    fn test_base_name_parsing() {
        assert_eq!(("random".to_string(), 0), get_base_name_and_width("random[0]".to_string()));
        assert_eq!(("random".to_string(), 0), get_base_name_and_width("random".to_string()));
        assert_eq!(("random".to_string(), 7), get_base_name_and_width("random[7]".to_string()));
        assert_eq!(("foo_".to_string(), 17), get_base_name_and_width("foo_[17]".to_string()));

    }
}
