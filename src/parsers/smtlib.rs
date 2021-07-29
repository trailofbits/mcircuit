use std::fs::File;
use std::io::BufReader;

use crate::parsers::{Parse, WireHasher};
use crate::Operation;

#[allow(dead_code)] // TODO: remove once implemented
struct SMTLibParser {
    reader: BufReader<File>,
    hasher: WireHasher,
}

impl Parse<bool> for SMTLibParser {
    type Item = Operation<bool>;

    fn new(reader: BufReader<File>) -> Self {
        SMTLibParser {
            reader,
            hasher: Default::default(),
        }
    }

    fn next(&mut self) -> Option<Operation<bool>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use lexpr::{Cons, Value};

    #[test]
    fn test_sample2() {
        let samp = "(declare-fun k!1200 () Bool)";
        let v = lexpr::from_str(samp).unwrap();
        match v {
            Value::Nil => {
                println!("Value::Nil: {}", v)
            }
            Value::Null => {
                println!("Value::Null: {}", v)
            }
            Value::Bool(_) => {
                println!("Value::Bool: {}", v)
            }
            Value::Number(_) => {
                println!("Value::Number: {}", v)
            }
            Value::Char(_) => {
                println!("Value::Char: {}", v)
            }
            Value::String(_) => {
                println!("Value::String: {}", v)
            }
            Value::Symbol(_) => {
                println!("Value::Symbol: {}", v)
            }
            Value::Keyword(_) => {
                println!("Value::Keyword: {}", v)
            }
            Value::Bytes(_) => {
                println!("Value::Bytes: {}", v)
            }
            Value::Cons(c) => {
                let (left, right) = c.as_pair();
                println!("Left: {}", left);
                println!("Right: {}", right);
            }
            Value::Vector(_) => {
                println!("Value::Vector: {}", v)
            }
        }
    }

    fn sample() -> String {
        "(declare-fun k!1200 () Bool)
(declare-fun k!1190 () Bool)
(declare-fun k!1180 () Bool)
(declare-fun k!1170 () Bool)
(declare-fun k!1160 () Bool)
(declare-fun k!1150 () Bool)
(declare-fun k!1140 () Bool)
(declare-fun k!1130 () Bool)
(declare-fun k!1120 () Bool)
(declare-fun k!1110 () Bool)
(declare-fun k!1100 () Bool)
(declare-fun k!1090 () Bool)
(declare-fun k!1080 () Bool)
(declare-fun k!1070 () Bool)
(declare-fun k!1060 () Bool)
(declare-fun k!1050 () Bool)
(declare-fun k!1040 () Bool)
(declare-fun k!1030 () Bool)
(declare-fun k!1020 () Bool)
(declare-fun k!1010 () Bool)
(declare-fun k!1000 () Bool)
(declare-fun k!990 () Bool)
(declare-fun k!980 () Bool)
(declare-fun k!970 () Bool)
(declare-fun k!960 () Bool)
(declare-fun k!950 () Bool)
(declare-fun k!940 () Bool)
(declare-fun k!930 () Bool)
(declare-fun k!920 () Bool)
(declare-fun k!910 () Bool)
(declare-fun k!900 () Bool)
(declare-fun k!890 () Bool)
(declare-fun k!880 () Bool)
(declare-fun k!870 () Bool)
(declare-fun k!860 () Bool)
(declare-fun k!850 () Bool)
(declare-fun k!840 () Bool)
(declare-fun k!830 () Bool)
(declare-fun k!820 () Bool)
(declare-fun k!810 () Bool)
(declare-fun k!800 () Bool)
(declare-fun k!790 () Bool)
(declare-fun k!780 () Bool)
(declare-fun k!770 () Bool)
(declare-fun k!760 () Bool)
(declare-fun k!750 () Bool)
(declare-fun k!740 () Bool)
(declare-fun k!730 () Bool)
(declare-fun k!720 () Bool)
(declare-fun k!710 () Bool)
(declare-fun k!700 () Bool)
(declare-fun k!690 () Bool)
(declare-fun k!680 () Bool)
(declare-fun k!670 () Bool)
(declare-fun k!660 () Bool)
(declare-fun k!650 () Bool)
(declare-fun k!640 () Bool)
(declare-fun k!630 () Bool)
(declare-fun k!620 () Bool)
(declare-fun k!610 () Bool)
(declare-fun k!600 () Bool)
(declare-fun k!590 () Bool)
(declare-fun k!580 () Bool)
(declare-fun k!570 () Bool)
(declare-fun k!560 () Bool)
(declare-fun k!550 () Bool)
(declare-fun k!540 () Bool)
(declare-fun k!530 () Bool)
(declare-fun k!520 () Bool)
(declare-fun k!510 () Bool)
(declare-fun k!500 () Bool)
(declare-fun k!490 () Bool)
(declare-fun k!480 () Bool)
(declare-fun k!470 () Bool)
(declare-fun k!460 () Bool)
(declare-fun k!450 () Bool)
(declare-fun k!440 () Bool)
(declare-fun k!430 () Bool)
(declare-fun k!420 () Bool)
(declare-fun k!410 () Bool)
(declare-fun k!400 () Bool)
(declare-fun k!390 () Bool)
(declare-fun k!380 () Bool)
(declare-fun k!370 () Bool)
(declare-fun k!360 () Bool)
(declare-fun k!350 () Bool)
(declare-fun k!340 () Bool)
(declare-fun k!330 () Bool)
(declare-fun k!320 () Bool)
(declare-fun k!310 () Bool)
(declare-fun k!300 () Bool)
(declare-fun k!290 () Bool)
(declare-fun k!280 () Bool)
(declare-fun k!270 () Bool)
(declare-fun k!260 () Bool)
(declare-fun k!200 () Bool)
(declare-fun k!240 () Bool)
(declare-fun k!220 () Bool)
(declare-fun k!210 () Bool)
(declare-fun k!90 () Bool)
(declare-fun k!160 () Bool)
(declare-fun k!70 () Bool)
(declare-fun k!120 () Bool)
(declare-fun k!80 () Bool)
(declare-fun k!150 () Bool)
(declare-fun k!140 () Bool)
(declare-fun k!130 () Bool)
(declare-fun k!110 () Bool)
(declare-fun k!250 () Bool)
(declare-fun k!230 () Bool)
(declare-fun k!180 () Bool)
(declare-fun k!100 () Bool)
(declare-fun k!00 () Bool)
(assert
 (let (($x403 (not k!1200)))
 (let (($x409 (not k!1190)))
 (let (($x411 (not k!1180)))
 (let (($x414 (not k!1170)))
 (let (($x415 (not k!1160)))
 (let (($x430 (not k!1150)))
 (let (($x589 (not k!1140)))
 (let (($x693 (not k!1130)))
 (let (($x755 (not k!1120)))
 (let (($x523 (not k!1110)))
 (let (($x489 (not k!1100)))
 (let (($x712 (not k!1090)))
 (let (($x696 (not k!1080)))
 (let (($x698 (not k!1070)))
 (let (($x699 (not k!1060)))
 (let (($x690 (not k!1050)))
 (let (($x462 (not k!1040)))
 (let (($x706 (not k!1030)))
 (let (($x730 (not k!1020)))
 (let (($x504 (not k!1010)))
 (let (($x517 (not k!1000)))
 (let (($x518 (not k!990)))
 (let (($x519 (not k!980)))
 (let (($x520 (not k!970)))
 (let (($x465 (not k!960)))
 (let (($x466 (not k!950)))
 (let (($x467 (not k!940)))
 (let (($x468 (not k!930)))
 (let (($x469 (not k!920)))
 (let (($x752 (not k!910)))
 (let (($x650 (not k!900)))
 (let (($x652 (not k!890)))
 (let (($x653 (not k!880)))
 (let (($x654 (not k!870)))
 (let (($x655 (not k!860)))
 (let (($x656 (not k!850)))
 (let (($x658 (not k!840)))
 (let (($x463 (not k!830)))
 (let (($x659 (not k!820)))
 (let (($x477 (not k!810)))
 (let (($x660 (not k!800)))
 (let (($x490 (not k!790)))
 (let (($x661 (not k!780)))
 (let (($x503 (not k!770)))
 (let (($x662 (not k!760)))
 (let (($x515 (not k!750)))
 (let (($x663 (not k!740)))
 (let (($x526 (not k!730)))
 (let (($x664 (not k!720)))
 (let (($x537 (not k!710)))
 (let (($x665 (not k!700)))
 (let (($x548 (not k!690)))
 (let (($x666 (not k!680)))
 (let (($x553 (not k!670)))
 (let (($x667 (not k!660)))
 (let (($x565 (not k!650)))
 (let (($x566 (not k!640)))
 (let (($x668 (not k!630)))
 (let (($x579 (not k!620)))
 (let (($x669 (not k!610)))
 (let (($x591 (not k!600)))
 (let (($x592 (not k!590)))
 (let (($x670 (not k!580)))
 (let (($x604 (not k!570)))
 (let (($x605 (not k!560)))
 (let (($x671 (not k!550)))
 (let (($x616 (not k!540)))
 (let (($x672 (not k!530)))
 (let (($x627 (not k!520)))
 (let (($x673 (not k!510)))
 (let (($x638 (not k!500)))
 (let (($x674 (not k!490)))
 (let (($x649 (not k!480)))
 (let (($x675 (not k!470)))
 (let (($x676 (not k!460)))
 (let (($x677 (not k!450)))
 (let (($x740 (not k!440)))
 (let (($x725 (not k!430)))
 (let (($x733 (not k!420)))
 (let (($x613 (not k!410)))
 (let (($x728 (not k!400)))
 (let (($x687 (not k!390)))
 (let (($x739 (not k!380)))
 (let (($x707 (not k!370)))
 (let (($x534 (not k!360)))
 (let (($x734 (not k!350)))
 (let (($x635 (not k!340)))
 (let (($x602 (not k!330)))
 (let (($x757 (not k!320)))
 (let (($x704 (not k!310)))
 (let (($x742 (not k!300)))
 (let (($x536 (not k!290)))
 (let (($x396 (not k!280)))
 (let (($x429 (not k!270)))
 (let (($x760 (not k!260)))
 (let (($x716 (not k!240)))
 (let (($x724 (not k!220)))
 (let (($x476 (not k!210)))
 (let (($x689 (not k!150)))
 (let (($x557 (not k!140)))
 (let (($x558 (not k!130)))
 (let (($x703 (not k!110)))
 (let (($x428 (not k!250)))
 (let (($x732 (not k!180)))
 (let (($x610 (not k!100)))
 (let (($x432 (not k!00)))
 (and $x432 $x610 $x732 k!230 $x428 true $x703 $x558 $x557 $x689 k!80 k!120 k!70 k!160 k!90 $x476 $x724 $x716 k!200 $x760 $x429 $x396 $x536 $x742 $x704 $x757 $x602 $x635 $x734 $x534 $x707 $x739 $x687 $x728 $x613 $x733 $x725 $x740 $x677 $x676 $x675 $x649 $x674 $x638 $x673 $x627 $x672 $x616 $x671 $x605 $x604 $x670 $x592 $x591 $x669 $x579 $x668 $x566 $x565 $x667 $x553 $x666 $x548 $x665 $x537 $x664 $x526 $x663 $x515 $x662 $x503 $x661 $x490 $x660 $x477 $x659 $x463 $x658 $x656 $x655 $x654 $x653 $x652 $x650 $x752 $x469 $x468 $x467 $x466 $x465 $x520 $x519 $x518 $x517 $x504 $x730 $x706 $x462 $x690 $x699 $x698 $x696 $x712 $x489 $x523 $x755 $x693 $x589 $x430 $x415 $x414 $x411 $x409 $x403))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
".into()
    }
}
