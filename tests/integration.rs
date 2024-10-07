use rstest::rstest;
use rusty_lox::{bytecode::Value, compiler::compile, vm::VM};

#[rstest]
#[case("1 + 2", Value::Double(3.0))]
#[case("(1 + 2)", Value::Double(3.0))]
#[case("-1", Value::Double(-1.0))]
#[case("(-1 + 2) * 3 - -4", Value::Double(7.0))]
#[case("2 * 3 + 4", Value::Double(10.0))]
#[case("2 + 3 * 4", Value::Double(14.0))]
#[case("(2 + 3) * 4", Value::Double(20.0))]
#[case("2 + 4 / 4", Value::Double(3.0))]
#[case("2 + 2 + 3 * 4", Value::Double(16.0))]
#[case("2 + 2 - 3 * 4", Value::Double(-8.0))]
#[case("true", Value::Bool(true))]
#[case("false", Value::Bool(false))]
#[case("nil", Value::Nil)]
#[case("!false", Value::Bool(true))]
#[case("!!false", Value::Bool(false))]
#[case("1 == 1", Value::Bool(true))]
#[case("1 != 2", Value::Bool(true))]
#[case("2 > 1", Value::Bool(true))]
#[case("2 > 2", Value::Bool(false))]
#[case("2 >= 2", Value::Bool(true))]
#[case("2 < 1", Value::Bool(false))]
#[case("2 < 2", Value::Bool(false))]
#[case("2 <= 2", Value::Bool(true))]
#[case("!(5 - 4 > 3 * 2 == !nil)", Value::Bool(true))]
#[case("\"x\"", Value::String("x".to_string()))]
#[case("\"x\" == \"x\"", Value::Bool(true))]
#[case("\"x\" == \"y\"", Value::Bool(false))]
#[case("\"x\" != \"y\"", Value::Bool(true))]
#[case("\"x\" + \"y\" == \"xy\"", Value::Bool(true))]
fn end_to_end(#[case] source: String, #[case] expected: Value) {
    let chunk = compile(&source).unwrap();
    let mut vm = VM::default();

    vm.interpret(&chunk).unwrap();
    assert_eq!(expected, vm.stack_top().unwrap().clone());
}
