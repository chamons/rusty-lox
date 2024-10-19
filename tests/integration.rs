use rstest::rstest;
use rusty_lox::{
    compiler::compile,
    vm::{InterpretErrors, VMSettings, VM},
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
#[case("true and false", "false")]
#[case("true and true", "true")]
#[case("true or true", "true")]
#[case("true or false", "true")]
#[case("false or true", "true")]
#[case("false or false", "false")]
fn end_to_end(#[case] source: String, #[case] expected: String) {
    let function = compile(&format!("print {source};")).unwrap();
    let mut vm = VM::new_from_settings(VMSettings::test_default());

    vm.interpret(function).unwrap();

    assert_eq!(expected, vm.captured_prints[0]);
}

#[rstest]
#[case(
    "var beverage = \"cafe au lait\";
var breakfast = \"beignets with \" + beverage;
print breakfast;",
    "beignets with cafe au lait"
)]
#[case(
    "var x = 1;
x = 2;
print x;",
    "2"
)]
#[case(
    "{
        var a = \"outer\";
        {
          var a = \"inner\";
          print a;
        }
      }",
    "inner"
)]
#[case(
    "var x = 1;
    if (x)
    {
        print x;
    }",
    "1"
)]
#[case(
    "var x = true;
    if (x)
    {
        print 1;
    }
    else
    {
        print 2;
    }",
    "1"
)]
#[case(
    "var x = false;
    if (x)
    {
        print 1;
    }
    else
    {
        print 2;
    }",
    "2"
)]
#[case(
    "if (true and false or true)
    {
        print 1;
    }",
    "1"
)]
#[case(
    "var output = \"\";
    var count = 0;
    if (count < 5)
    {
        count = count + 1;
        output = output + \"a\";
    }
    print output;",
    "a"
)]
#[case(
    "var output = \"\";
    var count = 0;
    while (count < 5)
    {
        count = count + 1;
        output = output + \"a\";
    }
    print output;",
    "aaaaa"
)]
#[case(
    "var count = 0;
    while (count < 500)
    {
        count = count + 1;
    }
    print \"done\";",
    "done"
)]
#[case(
    "var value = 0;
    for (var count = 0 ; count < 10 ; count = count + 1)
    {
        value = value + 1;
    }
    print value;",
    "10"
)]
#[case(
    "var value = 0;
    for (var count = 0 ; count < 10 ;)
    {
        count = count + 1;
        value = value + 1;
    }
    print value;",
    "10"
)]
#[case(
    "var value = 0;
    var count = 0;
    for (; count < 10 ;)
    {
        count = count + 1;
        value = value + 1;
    }
    print value;",
    "10"
)]
#[case(
    "fun f() {}
    print f;",
    "Function f"
)]
#[case(
    "fun first() {
  var a = 1;
  var b = 2;
  second(b);
}

fun second(b) {
  var c = 3;
  var d = 4;
  print b;
}

first();
",
    "2"
)]
#[case(
    "fun noReturn() {}
print noReturn();",
    "nil"
)]
#[case(
    "fun f() { return 42.0; }
print f();",
    "42"
)]
#[case(
    "fun f() { return 42.0; }
print f() + f();",
    "84"
)]
fn small_programs_end_to_end(#[case] source: String, #[case] expected: String) {
    println!("{}", source);

    let function = compile(&source).unwrap();

    println!("{}", function.chunk);

    let mut vm = VM::new_from_settings(VMSettings::test_default());

    vm.interpret(function).unwrap();
    assert_eq!(1, vm.captured_prints.len());
    assert_eq!(expected, vm.captured_prints[0]);
    assert!(vm.is_stack_empty());
}

#[rstest]
#[case(
    "var notAFunction = 123;
notAFunction();",
    InterpretErrors::InvalidRuntimeType
)]
#[case(
    "fun f(a) {}
f();",
    InterpretErrors::IncorrectArgumentCount(1, 0)
)]
fn small_programs_that_error(#[case] source: String, #[case] expected_error: InterpretErrors) {
    let function = compile(&source).unwrap();

    // println!("{}", function.chunk);

    let mut vm = VM::new_from_settings(VMSettings::test_default());

    let runtime_error = vm.interpret(function).unwrap_err();
    assert_eq!(runtime_error, expected_error);
}
