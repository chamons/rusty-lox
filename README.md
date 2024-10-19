# rusty-lox

This is an implementation of lox in rust through chapter 24 of [the book](https://craftinginterpreters.com/functions.html).

It contains:
- User declared functions, with recursion
- Built in timing function
- Basic addition and order of operation

with a bytecode compiler from the book ported from C to Rust.

## Show Me

```
% cat data/fib.lox                   
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

var start = clock();
print fib(35);
print clock() - start;

% cargo run -q --release -- data/fib.lox
9227465
3.6996841430664063
```