use std::convert::TryInto;
use std::io::{BufReader, Read, Result};

use crate::Witness;

#[allow(dead_code)]
fn parse_witness<const L: usize, R: Read>(source: &mut BufReader<R>) -> Result<Witness<L>> {
    let mut buf = String::new();
    source.read_to_string(&mut buf)?;

    let witness = buf
        .trim()
        .split('\n')
        .map(|step| {
            step.chars()
                .map(|c| match c {
                    '0' => false,
                    '1' => true,
                    _ => panic!("Bad bit {:?} in witness!", c),
                })
                .collect::<Vec<bool>>()
                .try_into()
                .expect("bad witness size!")
        })
        .collect();
    Ok(Witness { witness })
}

#[cfg(test)]
mod tests {
    use crate::parsers::witness::parse_witness;
    use std::io::BufReader;

    #[test]
    fn test_parse_witness() {
        let s = "0000\n1111\n0101\n1010\n";
        let mut br = BufReader::new(s.as_bytes());

        let wit = parse_witness::<4, &[u8]>(&mut br).expect("bad witness");

        assert_eq!(
            wit.witness,
            vec![
                [false, false, false, false],
                [true, true, true, true],
                [false, true, false, true],
                [true, false, true, false]
            ]
        )
    }
}
