use num_traits::Zero;
use std::collections::{VecDeque};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem::swap;
use std::mem::take;

use crate::parsers::{Parse, WireHasher};
use crate::WireValue;
use crate::{OpType, Operation};

/// Parses single wire pairs of the format `parent=child`. Returns (parent, child)
pub fn parse_split(pair: &str) -> (&str, &str) {
    let mut split = pair.split('=');
    (split.next().unwrap(), split.next().unwrap())
}

/// Parses a gate into an operand, an output wire, and input wires. Returns in that order.
fn parse_gate(mut line: VecDeque<&str>) -> (&str, &str, Vec<&str>) {
    let op = line.pop_front().unwrap();
    let (_, out) = parse_split(line.pop_back().unwrap());
    let inputs: Vec<&str> = line.drain(..).map(|part| parse_split(part).1).collect();

    (op, out, inputs)
}

/// Parses a line of inputs or outputs into individual wires, then individual bits.
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

/// Parses a subcircuit line into the name of the circuit, then the list of I/O connections.
fn parse_subcircuit(mut line: VecDeque<&str>) -> (&str, Vec<(&str, &str)>) {
    let name = line.pop_front().unwrap();
    let io: Vec<(&str, &str)> = line.drain(..).map(parse_split).collect();

    (name, io)
}

/// Splits up a wire that ends with a bit index (`input[3]`) into individual components (`("input", 3)`)
pub fn get_base_name_and_width(unparsed: &str) -> (String, usize) {
    let (base_name, after): (String, Option<&str>) = match unparsed.split_once('[') {
        None => (unparsed.into(), None),
        Some((before, after)) => (before.to_string(), Some(after)),
    };
    let idx = match after {
        None => 0,
        Some(after) => after
            .split_once(']')
            .unwrap()
            .0
            .parse::<usize>()
            .expect(after),
    };
    (base_name, idx)
}

/// Returns `{context}::{id}`. Double colon syntax is used by the VCD dumper to separate scopes.
/// Ignores `$true` and `$false` and rejects `$undef`.
pub fn format_wire_id(context: &str, id: &str) -> String {
    if (id == "$true") || (id == "$false") {
        id.to_string()
    } else if id == "$undef" {
        panic!("{} contains an $undef wire", context);
    } else {
        format!("{}::{}", context, id)
    }
}

/// A set of data that represents the information about a circuit we can glean from the BLIF file.
/// May have multiple circuits per file.
#[derive(Clone)]
pub struct BlifCircuitDesc<T: WireValue> {
    pub name: String,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub gates: Vec<Operation<T>>,
    pub subcircuits: Vec<BlifSubcircuitDesc>,
}

/// Defines the relation between a circuit and its subcircuits
#[derive(Clone)]
pub struct BlifSubcircuitDesc {
    pub name: String,
    /// A set of wire ID connections in the format `(parent, subcircuit)`
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

impl<T: WireValue> BlifCircuitDesc<T> {
    /// Just pushes to `self.subcircuit`. Used to do packed wire expansion but that's handled
    /// elsewhere now.
    fn add_subcircuit(&mut self, sub: BlifSubcircuitDesc) {
        self.subcircuits.push(sub)
    }

    /// Checks that input and output wires are contiguous blocks, which they _should_ be in the
    /// top-level circuit after the hashing process. Later called by the flattener on the top-level
    /// circuit. It doesn't necessarily have to be true for anything but the top-level.
    pub fn validate_io(&self) {
        if let Some(max_input) = self.inputs.iter().max() {
            let min_input = self.inputs.iter().min().unwrap();

            if (max_input - min_input) != (self.inputs.len() - 1) {
                panic!(
                    "{}'s inputs are not contiguous!\n{:?}",
                    self.name, self.inputs
                )
            }
        }

        if let Some(max_output) = self.outputs.iter().max() {
            let min_output = self.outputs.iter().min().unwrap();

            if (max_output - min_output) != (self.outputs.len() - 1) {
                panic!(
                    "{}'s outputs are not contiguous!\n{:?}",
                    self.name, self.outputs
                )
            }
        }
    }
}

/// This trait lets us introduce some genericity into the parsing process. We can construct boolean
/// and u64 variants of gates. Results in some very similar code, but we need it since the names of
/// gates in an arithmetic context and boolean context are different.
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
    /// Vector - can have more than one circuit descriptor per file.
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

/// Translates tokens into boolean gates
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

/// Translates tokens into arithmetic gates
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

/// Breaks up wires that contain `_PACKED_<width>` into `<width>` bits. Uglier than the old `.attr`
/// technique, but much easier to develop and debug because the packing is part of the wire name,
/// whereas with attributes you don't find out that a wire is packed until _after_ you've parsed it.
/// You end up needing to save a lot of things until after you've parsed the next several lines, _then_
/// parse them all at once, which gets complicated.
pub fn split_wire_id(id: &str) -> Vec<String> {
    if id.contains("_PACKED_") {
        let (base, idx) = get_base_name_and_width(id);
        match base.split_once("_PACKED_") {
            None => {
                unreachable!("Already did .contains!")
            }
            Some((name, width_dec)) => {
                let width: usize = width_dec
                    .parse()
                    .unwrap_or_else(|_| panic!("Can't parse {} as an integer", width_dec));
                // If we ever add endianness information to packed wire names, you could throw a
                // `.rev()` in here
                (0..width)
                    // Multiply the current index by the width of the wire, then add the current bit
                    // index.
                    .map(|i| format!("{}[{}]", name, (width * idx) + i))
                    .collect()
            }
        }
    } else {
        // I'd love to figure out how to return an iterator here, but since something needs to own
        // the formatted strings in the other case, I'm not sure it's possible. If only we could
        // partially apply arguments to `format`...
        vec![id.to_string()]
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
                        // Break up the I/O line into chunks for each wire
                        for chunk in parse_io(line) {
                            // Yosys gives us the wire IDs in descending order in MSP430 because the
                            // top-level circuit uses [lo:hi] for indexing. With packed wires, this
                            // shouldn't matter.
                            for name_maybe_packed in chunk.iter().rev() {
                                // Split the wire ID into multiple (if it's packed)
                                for name in split_wire_id(name_maybe_packed) {
                                    // Format it with the current module name
                                    let formatted = format_wire_id(&current.name, &name);
                                    // Take the hash and save it.
                                    current.inputs.push(self.hasher.get_wire_id(&formatted));
                                }
                            }
                        }
                    }
                    ".outputs" => {
                        for chunk in parse_io(line) {
                            for name_maybe_packed in chunk.iter().rev() {
                                for name in split_wire_id(name_maybe_packed) {
                                    let formatted = format_wire_id(&current.name, &name);
                                    current.outputs.push(self.hasher.get_wire_id(&formatted));
                                }
                            }
                        }
                    }
                    ".gate" => {
                        let (op, out, mut inputs) = parse_gate(line);
                        // get the output
                        let out_id = self.hasher.get_wire_id(&format_wire_id(&current.name, out));
                        // get the inputs
                        let input_ids: Vec<usize> = inputs
                            .drain(..)
                            .map(|name| {
                                self.hasher
                                    .get_wire_id(&format_wire_id(&current.name, name))
                            })
                            .collect();
                        // Turn the strings and wire IDs into an `Operation`
                        current
                            .gates
                            .push(self.construct_variant(op, out_id, &input_ids, None));
                    }
                    ".subckt" => {
                        let (name, mut io_pairings) = parse_subcircuit(line);
                        let mut connections: Vec<(usize, usize)> = Vec::new();
                        for (child_name, parent_name) in io_pairings.drain(..) {
                            // Split both the parent and child connections if they're both packed
                            let child_unpacked = split_wire_id(child_name);
                            let mut parent_unpacked = split_wire_id(parent_name);

                            if child_unpacked.len() != parent_unpacked.len() {
                                // We can handle packed wires that connect to const gates by just
                                // duplicating the connection
                                if parent_name == "$false" || parent_name == "$true" {
                                    parent_unpacked =
                                        vec![parent_name.into(); child_unpacked.len()];
                                }
                                // but any other time we have a mismatch in sizes, it's not clear
                                // what to do
                                else {
                                    panic!(
                                        "{} expanded to {} bits, but {} expanded to {} bits",
                                        child_name,
                                        child_unpacked.len(),
                                        parent_name,
                                        parent_unpacked.len()
                                    );
                                }
                                // I mean maybe if one wire is packed and the other is a single bit,
                                // we could expand the single wire, but we haven't needed that yet.
                            }

                            // Does the `rev` on `parent_unpacked` seem weird to you? Well, it should! If a subcircuit wire uses one index convention
                            // ([hi: lo]) and the parent wire uses another ([lo:hi]), Yosys will expect that the bit indices are inverted when
                            // hooking up the subcircuit. For that reason, we swap around the parent wires and use descending order.
                            // This won't always be the case. In the MSP430 circuit, all the wires in the top-level circuit use the same
                            // convention, and all the wires in the subcircuits use the same (opposite) convention, so universal inverting works
                            // fine here. If you use the same convention in the top-level as the subcircuits, you'll need to flip this around. If you
                            // mix and match conventions between different subcircuits, it won't work _at all_ because we don't annotate packed wires
                            // with an ordering convention.

                            // Hopefully I remembered to document this somewhere else too. If not, sorry. At least now you know...
                            for (cname, pname) in
                                child_unpacked.iter().zip(parent_unpacked.iter().rev())
                            {
                                connections.push((
                                    self.hasher
                                        .get_wire_id(&format_wire_id(&current.name, pname)),
                                    self.hasher.get_wire_id(&format_wire_id(name, cname)),
                                ));
                            }
                        }

                        let subc = BlifSubcircuitDesc {
                            name: name.into(),
                            connections,
                        };

                        current.add_subcircuit(subc);
                    }
                    // These lines shouldn't be generated using the Yosys settings we've chosen, so if you see them, maybe
                    // double check that the undersigned logic is actually correct.
                    ".names" | ".conn" => {
                        let from = self
                            .hasher
                            .get_wire_id(&format_wire_id(&current.name, line.pop_front().unwrap()));
                        let to = self
                            .hasher
                            .get_wire_id(&format_wire_id(&current.name, line.pop_back().unwrap()));
                        current
                            .gates
                            .push(self.construct_variant("BUF", to, &[from], None))
                    }
                    ".end" => {
                        self.circuit.push(take(&mut current));
                        // Push const gates for true & false to the new circuit
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
                    _ => (),
                }
            }
        }
    }

    /// Parse the previous file and prepare to parse the next one on a subsequent call to `next`.
    /// This lets us split up a circuit across multiple BLIF files for simplicity.
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
        if !self.circuit.is_empty() {
            Some(self.circuit.remove(0))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::parsers::blif::{
        get_base_name_and_width, parse_gate, parse_io, parse_subcircuit, split_wire_id,
    };

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
        assert_eq!(
            ("random".to_string(), 0),
            get_base_name_and_width("random[0]")
        );
        assert_eq!(("random".to_string(), 0), get_base_name_and_width("random"));
        assert_eq!(
            ("random".to_string(), 7),
            get_base_name_and_width("random[7]")
        );
        assert_eq!(
            ("foo_".to_string(), 17),
            get_base_name_and_width("foo_[17]")
        );
        assert_eq!(
            ("std::fake::test".to_string(), 0),
            get_base_name_and_width("std::fake::test[0]")
        );
    }

    #[test]
    fn test_packed_wire_split() {
        assert_eq!(
            split_wire_id("foobar_PACKED_2[0]"),
            vec!["foobar[0]".to_string(), "foobar[1]".to_string(),]
        );

        assert_eq!(
            split_wire_id("foobar_PACKED_4[3]"),
            vec![
                "foobar[12]".to_string(),
                "foobar[13]".to_string(),
                "foobar[14]".to_string(),
                "foobar[15]".to_string(),
            ]
        );
        assert_eq!(
            split_wire_id("foobar_PACKED_3"),
            vec![
                "foobar[0]".to_string(),
                "foobar[1]".to_string(),
                "foobar[2]".to_string(),
            ]
        );

        assert_eq!(split_wire_id("foobar_[3]"), vec!["foobar_[3]"]);

        assert_eq!(split_wire_id("foobar_PA"), vec!["foobar_PA"]);
    }
}
