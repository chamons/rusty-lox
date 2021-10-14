# rusty-lox

This is an implementation of lox in rust through chapter 10 of [the book](https://craftinginterpreters.com/functions.html).

It contains:
- User declared functions, with recursion
- Built in timing function
- Basic addition and order of operation

with a recursive descent parser from the book ported from Java to Rust.

## Show Me

```
$ cargo run --release fib.lox
fib(30):
832040
Time:
15.474133014678955
```

This was a solo hackathon project for the year.
