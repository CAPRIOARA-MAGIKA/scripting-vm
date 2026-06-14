# scripting-vm

A small, complete, Lox-style dynamic scripting language with a stack-based bytecode VM. Written in Rust. Portfolio piece.

## About this project

I built this end-to-end as a portfolio piece to demonstrate systems-level thinking in Rust. The goal was a complete language pipeline -- not a toy parser, not a half-finished interpreter, but a working compiler and VM that runs real programs, with the kind of test discipline you'd expect from production code.

**The two-executors idea** is what I'm proudest of. The tree-walking interpreter and the bytecode VM are independently built. The end-to-end test suite (`tests/parity.rs`) runs every test program through *both* paths and asserts they produce identical output. When they diverge, that's a bug -- and the harness catches it on every test run. This is how the [Crafting Interpreters](https://craftinginterpreters.com/) book builds the case for a reference implementation, and it's the most useful piece of engineering infrastructure in the whole project.

**What I got out of it:**
- Hands-on with Rust's borrow checker on a non-trivial project (`Rc<RefCell<…>>` upvalues, lifetime juggling in the compiler's enclosing-function chain)
- Implementing closures correctly -- open vs closed upvalues, the trick of hoisting captured locals to the heap when the enclosing frame pops
- Single-pass compilation: no IR, no register allocation, no peephole -- just `AST → bytecode → execute`
- A maintainable test pyramid: lexer/parser unit tests, smoke tests at every layer, and a parity suite that pins the VM to the reference

**What's deliberately not here:** classes, a tracing garbage collector, a real standard library, optimization passes. v1 was scoped to ship a working core; v2 items are listed in `docs/known-limitations.md` with honest tradeoffs, not glossed over.

The project is single-author. Every commit is mine. The structure (Cargo project, modular Rust, 53 tests, four docs) is what I'd want a reviewer to see.

## What it is

A from-scratch implementation of a language pipeline: source → tokens → AST → bytecode → execution. Two execution paths exist -- a tree-walking reference and the bytecode VM -- and an end-to-end parity test suite asserts they agree on every program.

## What's implemented

- Lexer with line tracking
- Recursive-descent parser with Pratt precedence
- Tree-walking interpreter (reference)
- Single-pass AST → bytecode compiler
- Stack-based VM with closures and upvalues
- Dynamic typing, first-class functions, recursion
- REPL and `run` subcommand
- Parity test suite: 21 `.lang` programs run through both paths, output must match

## What's not implemented (v1)

- Classes, inheritance, `this`, `super`
- Tracing garbage collector (uses `Rc`; cycles leak -- see [known limitations](docs/known-limitations.md))
- Standard library beyond `print` and `clock`
- Optimization passes
- Debug protocol

## Build & run

```bash
cargo build --release
cargo run -- run examples/hello.lang
cargo run -- repl
```

## Run tests

```bash
cargo test
```

The parity test (`tests/parity.rs`) is the key check -- when both executors produce identical output on all 21 programs, the VM matches the reference.

## Project layout

```
src/
  lexer.rs       source -> tokens
  parser.rs      tokens -> AST
  ast.rs         AST types
  value.rs       runtime values
  obj.rs         heap objects
  env.rs         lexical scopes
  interpreter.rs tree-walking reference
  compiler.rs    AST -> bytecode
  opcode.rs      opcode set + Chunk
  upvalue.rs     closure capture
  vm.rs          stack machine
  native.rs      built-in functions
  main.rs        CLI
tests/cases/*.lang  parity test programs
```

## Architecture

See [docs/architecture.md](docs/architecture.md). Opcode reference: [docs/bytecode.md](docs/bytecode.md).

## Disclaimer

This project was designed and built strictly for educational and portfolio purposes; it is not intended for commercial use. To ensure high quality, an LLM was utilized for code review, test design, and polishing the documentation.

## Example

```rust
fn fib(n) {
  if (n < 2) { return n; }
  return fib(n - 1) + fib(n - 2);
}
print fib(10);  // 55
```

```rust
fn makeCounter() {
  var i = 0;
  fn tick() {
    i = i + 1;
    return i;
  }
  return tick;
}
var c = makeCounter();
print c();  // 1
print c();  // 2
print c();  // 3
```
