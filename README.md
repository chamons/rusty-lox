# rusty-lox

This is an implementation of lox in rust through chapter 10 of [the book](https://craftinginterpreters.com/functions.html).

It contains:
- User declared functions, with recursion
- Built in timing function
- Basic addition and order of operation

with a recursive descent parser from the book ported from Java to Rust.

## Show Me

```
$ cat fib.lox 
fun fib(n) {
    if (n <= 1) return n;
    return fib(n - 2) + fib(n - 1);
}
var start = clock();
print "fib(30):";
print fib(30);
var end = clock();
print "Time:";
print end - start;

$ cargo run --release fib.lox
fib(30):
832040
Time:
15.474133014678955
```

This was a solo hackathon project for the year.
