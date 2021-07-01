#[cfg(test)]
mod tests {
    use crate::{CombineOperation, HasIO, OpType, Operation};
    use rand::thread_rng;

    #[test]
    fn test_io_operations() {
        fn check_combine_gf2(
            gate: Operation<bool>,
            collected_inputs: Vec<usize>,
            collected_outputs: Vec<usize>,
        ) {
            let as_combine = CombineOperation::GF2(gate);
            let collected_inputs_combine: Vec<usize> = as_combine.inputs().collect();
            let collected_outputs_combine: Vec<usize> = as_combine.outputs().collect();

            assert_eq!(collected_inputs, collected_inputs_combine);
            assert_eq!(collected_outputs, collected_outputs_combine);
        }

        fn check_combine_u64(
            gate: Operation<u64>,
            collected_inputs: Vec<usize>,
            collected_outputs: Vec<usize>,
        ) {
            let as_combine = CombineOperation::Z64(gate);
            let collected_inputs_combine: Vec<usize> = as_combine.inputs().collect();
            let collected_outputs_combine: Vec<usize> = as_combine.outputs().collect();

            assert_eq!(collected_inputs, collected_inputs_combine);
            assert_eq!(collected_outputs, collected_outputs_combine);
        }

        for _ in 0..1000 {
            // Test for GF2
            match Operation::<bool>::random_variant(&mut thread_rng()) {
                OpType::BinaryOp(ty) => {
                    let (out, in1, in2): (usize, usize, usize) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1, in2]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_gf2(gate, collected_inputs, collected_outputs);
                }
                OpType::BinaryConstOp(ty) => {
                    let (out, in1, in2): (usize, usize, bool) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_gf2(gate, collected_inputs, collected_outputs);
                }
                OpType::InputOp(ty) => {
                    let out: usize = rand::random();
                    let gate = ty(out);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_gf2(gate, collected_inputs, collected_outputs);
                }
                OpType::InputConstOp(ty) => {
                    let (out, in1): (usize, bool) = rand::random();
                    let gate = ty(out, in1);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_gf2(gate, collected_inputs, collected_outputs);
                }
                OpType::OutputConstOp(ty) => {
                    let (in1, in2): (usize, bool) = rand::random();
                    let gate = ty(in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![]);

                    check_combine_gf2(gate, collected_inputs, collected_outputs);
                }
            }

            // Test for Z64
            match Operation::<u64>::random_variant(&mut thread_rng()) {
                OpType::BinaryOp(ty) => {
                    let (out, in1, in2): (usize, usize, usize) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1, in2]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_u64(gate, collected_inputs, collected_outputs);
                }
                OpType::BinaryConstOp(ty) => {
                    let (out, in1, in2): (usize, usize, u64) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_u64(gate, collected_inputs, collected_outputs);
                }
                OpType::InputOp(ty) => {
                    let out: usize = rand::random();
                    let gate = ty(out);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_u64(gate, collected_inputs, collected_outputs);
                }
                OpType::InputConstOp(ty) => {
                    let (out, in1): (usize, u64) = rand::random();
                    let gate = ty(out, in1);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine_u64(gate, collected_inputs, collected_outputs);
                }
                OpType::OutputConstOp(ty) => {
                    let (in1, in2): (usize, u64) = rand::random();
                    let gate = ty(in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![]);

                    check_combine_u64(gate, collected_inputs, collected_outputs);
                }
            }
        }
    }

    #[test]
    fn test_io_combine_operations() {
        // GF2/Z64 are handled by the previous test

        for _ in 0..10 {
            // Test B2A
            let (out, low): (usize, usize) = rand::random();
            let gate = CombineOperation::B2A(out, low);

            let mut expected_inputs: Vec<usize> = vec![];
            for i in low..(low + 64) {
                expected_inputs.push(i);
            }
            let collected_inputs: Vec<usize> = gate.inputs().collect();
            let collected_outputs: Vec<usize> = gate.outputs().collect();
            assert_eq!(collected_inputs, expected_inputs);
            assert_eq!(collected_outputs, vec![out]);

            // Test SizeHint
            let gate = CombineOperation::SizeHint(out, low);
            let collected_inputs: Vec<usize> = gate.inputs().collect();
            let collected_outputs: Vec<usize> = gate.outputs().collect();
            assert_eq!(collected_inputs, vec![]);
            assert_eq!(collected_outputs, vec![]);
        }
    }
}
