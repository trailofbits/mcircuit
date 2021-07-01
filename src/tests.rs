#[cfg(test)]
mod tests {
    use crate::{CombineOperation, HasIO, OpType, Operation, Translatable, WireValue};
    use rand::distributions::{Distribution, Standard};
    use rand::thread_rng;

    #[test]
    fn test_io_operations() {
        fn check_combine<T: WireValue>(
            gate: Operation<T>,
            collected_inputs: Vec<usize>,
            collected_outputs: Vec<usize>,
        ) where
            CombineOperation: From<Operation<T>>,
        {
            let as_combine: CombineOperation = gate.into();
            let collected_inputs_combine: Vec<usize> = as_combine.inputs().collect();
            let collected_outputs_combine: Vec<usize> = as_combine.outputs().collect();

            assert_eq!(collected_inputs, collected_inputs_combine);
            assert_eq!(collected_outputs, collected_outputs_combine);
        }

        fn do_gate_test<T: WireValue>()
        where
            Standard:
                Distribution<usize> + Distribution<(usize, T)> + Distribution<(usize, usize, T)>,
            CombineOperation: From<Operation<T>>,
        {
            match Operation::<T>::random_variant(&mut thread_rng()) {
                OpType::BinaryOp(ty) => {
                    let (out, in1, in2): (usize, usize, usize) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1, in2]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine::<T>(gate, collected_inputs, collected_outputs);
                }
                OpType::BinaryConstOp(ty) => {
                    let (out, in1, in2): (usize, usize, T) = rand::random();
                    let gate = ty(out, in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine::<T>(gate, collected_inputs, collected_outputs);
                }
                OpType::InputOp(ty) => {
                    let out: usize = rand::random();
                    let gate = ty(out);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine::<T>(gate, collected_inputs, collected_outputs);
                }
                OpType::InputConstOp(ty) => {
                    let (out, in1): (usize, T) = rand::random();
                    let gate = ty(out, in1);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![]);
                    assert_eq!(collected_outputs, vec![out]);

                    check_combine::<T>(gate, collected_inputs, collected_outputs);
                }
                OpType::OutputConstOp(ty) => {
                    let (in1, in2): (usize, T) = rand::random();
                    let gate = ty(in1, in2);
                    let collected_inputs: Vec<usize> = gate.inputs().collect();
                    let collected_outputs: Vec<usize> = gate.outputs().collect();
                    assert_eq!(collected_inputs, vec![in1]);
                    assert_eq!(collected_outputs, vec![]);

                    check_combine::<T>(gate, collected_inputs, collected_outputs);
                }
            }
        }

        for _ in 0..1000 {
            do_gate_test::<bool>();
            do_gate_test::<u64>();
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

    #[test]
    fn test_translation_operations() {
        fn do_gate_test<T: WireValue>()
        where
            CombineOperation: From<Operation<T>>,
            Standard: Distribution<(usize, usize, usize, T)>,
        {
            let variant = Operation::<T>::random_variant(&mut thread_rng());
            let (original_out, original_in1, original_in2, original_c): (usize, usize, usize, T) =
                rand::random();
            let (translated_out, translated_in1, translated_in2, _new_c): (usize, usize, usize, T) =
                rand::random();

            let gate = Operation::<T>::construct(
                variant,
                [original_in1, original_in2].iter().copied(),
                [original_out].iter().copied(),
                Some(original_c),
            );

            let translation_target = Operation::<T>::construct(
                variant,
                [translated_in1, translated_in2].iter().copied(),
                [translated_out].iter().copied(),
                Some(original_c),
            );

            let identity = gate
                .translate(gate.inputs(), gate.outputs())
                .expect("Failed to perform identity translation");
            let translated = gate
                .translate(translation_target.inputs(), translation_target.outputs())
                .expect("Failed to perform actual translation");

            assert_eq!(gate, identity);
            assert_eq!(translation_target, translated);

            let as_combine: CombineOperation = gate.into();
            let target_as_combine: CombineOperation = translation_target.into();

            let identity_combine = as_combine
                .translate(as_combine.inputs(), as_combine.outputs())
                .unwrap();
            let translated_combine = as_combine
                .translate(target_as_combine.inputs(), target_as_combine.outputs())
                .unwrap();

            assert_eq!(as_combine, identity_combine);
            assert_eq!(target_as_combine, translated_combine);
        }

        for _ in 0..1000 {
            do_gate_test::<bool>();
            do_gate_test::<u64>();
        }
    }

    #[test]
    fn test_translation_combine_operations() {
        // GF2/Z64 are handled by the previous test

        for _ in 0..10 {
            // Test B2A
            let (out, low): (usize, usize) = rand::random();
            let (target_out, target_low): (usize, usize) = rand::random();

            let gate = CombineOperation::B2A(out, low);
            let translation_target = CombineOperation::B2A(target_out, target_low);

            let identity = gate.translate(gate.inputs(), gate.outputs()).unwrap();
            let translated = gate
                .translate(translation_target.inputs(), translation_target.outputs())
                .unwrap();

            assert_eq!(gate, identity);
            assert_eq!(translated, translation_target);

            // Test SizeHint
            let gate = CombineOperation::SizeHint(out, low);
            let translation_target = CombineOperation::SizeHint(target_out, target_low);

            let identity = gate.translate(gate.inputs(), gate.outputs()).unwrap();
            let translated = gate
                .translate(translation_target.inputs(), translation_target.outputs())
                .unwrap();

            assert_eq!(gate, identity);
            assert_eq!(gate, translated); // Translation should not change SizeHints
        }
    }
}
