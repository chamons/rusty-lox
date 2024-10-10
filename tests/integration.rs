use rstest::rstest;
use rusty_lox::{
    compiler::compile,
    vm::{VMSettings, VM},
};

#[rstest]
#[case("1 + 2", "3")]
#[case("(1 + 2)", "3")]
#[case("-1", "-1")]
#[case("(-1 + 2) * 3 - -4", "7")]
#[case("2 * 3 + 4", "10")]
#[case("2 + 3 * 4", "14")]
#[case("(2 + 3) * 4", "20")]
#[case("2 + 4 / 4", "3")]
#[case("2 + 2 + 3 * 4", "16")]
#[case("2 + 2 - 3 * 4", "-8")]
#[case("true", "true")]
#[case("false", "false")]
#[case("nil", "nil")]
#[case("!false", "true")]
#[case("!!false", "false")]
#[case("1 == 1", "true")]
#[case("1 != 2", "true")]
#[case("2 > 1", "true")]
#[case("2 > 2", "false")]
#[case("2 >= 2", "true")]
#[case("2 < 1", "false")]
#[case("2 < 2", "false")]
#[case("2 <= 2", "true")]
#[case("!(5 - 4 > 3 * 2 == !nil)", "true")]
#[case("\"x\"", "x")]
#[case("\"x\" == \"x\"", "true")]
#[case("\"x\" == \"y\"", "false")]
#[case("\"x\" != \"y\"", "true")]
#[case("\"x\" + \"y\" == \"xy\"", "true")]
#[case("\"x\" + \"y\" == \"xy\"", "true")]
fn end_to_end(#[case] source: String, #[case] expected: String) {
    let chunk = compile(&format!("print {source};")).unwrap();
    let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });

    vm.interpret(&chunk).unwrap();

    assert_eq!(expected, vm.captured_prints[0]);
}

#[rstest]
#[case(
    "var beverage = \"cafe au lait\";
var breakfast = \"beignets with \" + beverage;
print breakfast;",
    "beignets with cafe au lait"
)]
fn small_programs_end_to_end(#[case] source: String, #[case] expected: String) {
    let chunk = compile(&source).unwrap();
    let mut vm = VM::new_from_settings(VMSettings { capture_prints: true });

    vm.interpret(&chunk).unwrap();

    assert_eq!(expected, vm.captured_prints[0]);
}
