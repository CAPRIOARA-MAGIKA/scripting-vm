# Custom Scripting Language & Bytecode VM — Design Spec

**Date:** 2026-06-14
**Status:** Approved
**Target:** Portfolio piece. Single-author, public repo.

## 1. Goals

Build a small, complete, Lox-style dynamic scripting language with a stack-based bytecode VM, written in Rust. Demonstrates: lexical analysis, recursive-descent parsing, AST design, bytecode compilation, and a hand-written VM with closures.

**Non-goals (v1):** classes, inheritance, tracing GC, standard library beyond `print` and `clock`, optimization passes, debug protocol, JIT.

## 2. Language Surface

### 2.1 Value types

- `number` — IEEE-754 f64
- `string` — heap-allocated, immutable
- `bool` — `true` / `false`
- `nil`
- `function` — user-defined; first-class

### 2.2 Statements

`expr`, `var` (declaration), `block`, `if`/`else`, `while`, `return`, `fn` (declaration). Statements appear at top level and inside blocks.

### 2.3 Expressions

Literals, binary operators (`+ - * /`), unary (`- !`), variable, assignment, call, grouping `(...)`, logical `and` / `or`, comma operator (right-associative in `(a, b)`).

### 2.4 Sample programs

```
// Fibonacci
fn fib(n) {
  if n < 2 { return n; }
  return fib(n - 1) + fib(n - 2);
}
print fib(10);   // 55
```

```
// Closure capture
fn makeCounter() {
  let i = 0;
  fn tick() {
    i = i + 1;
    return i;
  }
  return tick;
}
let c = makeCounter();
print c();   // 1
print c();   // 2
```

### 2.5 Reserved keywords

`and`, `break`, `continue`, `class`, `else`, `false`, `fn`, `for`, `if`, `nil`, `or`, `return`, `super`, `this`, `true`, `var`, `while`. (Many are unused in v1; reserved for forward compatibility.)

## 3. Architecture

```
  source  ──>  Lexer          source code -> Vec<Token>
  tokens  ──>  Parser         tokens      -> Ast (Program)
  ast     ──>  Interpreter    ast         -> stdout             (reference impl)
  ast     ──>  Compiler       ast         -> Chunk (bytecode)
  chunk   ──>  VM             runs the bytecode stack machine
```

### 3.1 Two executors

The tree-walking interpreter is the **reference implementation**. The VM is the **primary execution path** (per project title). An end-to-end test suite runs every `.lang` test case through both paths and asserts identical stdout. This catches drift.

## 4. Module Layout

```
src/
  main.rs            CLI: repl | run <file> | compile <file>
  token.rs           Token, TokenType
  lexer.rs           source -> Vec<Token>, with line/column tracking
  ast.rs             Expr enum, Stmt enum, visitor pattern
  parser.rs          recursive descent, Pratt for expressions
  error.rs           CompileError, RuntimeError, with source spans
  value.rs           Value enum (Number, Bool, Nil, Obj) + Rc-based Obj handle
  obj.rs             ObjKind: String, Function, Closure, Upvalue
  env.rs             Environment (variable scopes, parent chain)
  interpreter.rs     tree-walking reference implementation
  opcode.rs          OpCode enum, Chunk struct, line-info table
  compiler.rs        AST -> bytecode (function-level compilation)
  upvalue.rs         Upvalue tracking for closures
  vm.rs              dispatch loop, call frames, operand stack
  native.rs          built-in natives: print, clock
tests/
  common/
    run_both.rs      harness: run .lang file through both paths, diff stdout
  cases/             .lang source files covering features
examples/
  fib.lang
  closure.lang
  fizzbuzz.lang
```

### 4.1 Module boundaries

- `value.rs` defines *what a value is*. `vm.rs` defines *how values are operated on*. They share only through `obj.rs`.
- `compiler.rs` does not import from `vm.rs`. They share types only through `opcode.rs` and `value.rs`.
- `interpreter.rs` is independent of `vm.rs` and `compiler.rs`. Both consume `ast.rs`.

## 5. Value & Memory Model

- `Value` is a tagged enum: `Number(f64) | Bool(bool) | Nil | Obj(Rc<RefCell<Obj>>)`.
- `Obj` carries `ObjKind: String | Function | Closure | Upvalue`.
- **GC strategy (v1):** reference counting via `Rc`. Limitation: cycles leak. Mitigation: the language has no mutable fields on heap objects; only `let` rebindings. This is documented, not a hack. A future `Gc<T>` newtype can be swapped in without changing call sites.

## 6. Bytecode Format

A `Chunk`:

- `code: Vec<u8>` — opcode stream
- `constants: Vec<Value>` — constant pool
- `lines: Vec<usize>` — one entry per byte of `code`, for error reporting

Encoding:

- Opcodes: 1 byte
- Operands: 1 byte (slot index, constant index, upvalue index) or 2 bytes (jump offset)
- Stack slots per frame: 256 max (1-byte index)
- Jumps: 2-byte signed relative offset, ±32k range

### 6.1 Opcode set (v1)

| Opcode          | Operands    | Stack effect | Description |
|-----------------|-------------|--------------|-------------|
| `CONST`         | u8 idx      | `... -> v`   | push `constants[idx]` |
| `NIL`           |             | `... -> nil` | push nil |
| `TRUE`          |             | `... -> true`| push true |
| `FALSE`         |             | `... -> false`| push false |
| `POP`           |             | `v -> ...`   | discard top |
| `GET_LOCAL`     | u8 slot     | `... -> v`   | push local at slot |
| `SET_LOCAL`     | u8 slot     | `v -> ...`   | pop and store to local |
| `GET_GLOBAL`    | u8 name     | `... -> v`   | push global |
| `DEFINE_GLOBAL` | u8 name     | `v -> ...`   | pop, define global |
| `SET_GLOBAL`    | u8 name     | `v -> ...`   | pop, assign global |
| `GET_UPVALUE`   | u8 idx      | `... -> v`   | push upvalue (see also `CLOSURE`) |
| `SET_UPVALUE`   | u8 idx      | `v -> ...`   | pop, assign upvalue |
| `ADD`           |             | `a b -> r`   | numeric or string concat |
| `SUB`           |             | `a b -> r`   | numeric |
| `MUL`           |             | `a b -> r`   | numeric |
| `DIV`           |             | `a b -> r`   | numeric |
| `NEG`           |             | `v -> r`     | numeric negation |
| `NOT`           |             | `v -> r`     | logical not |
| `EQUAL`         |             | `a b -> r`   | `==` (structural) |
| `GREATER`       |             | `a b -> r`   | numeric `>` |
| `LESS`          |             | `a b -> r`   | numeric `<` |
| `PRINT`         |             | `v -> ...`   | pop and call `print` |
| `JUMP`          | u16 offset  |              | unconditional forward |
| `JUMP_IF_FALSE` | u16 offset  | `v`          | pop, jump if false |
| `LOOP`          | u16 offset  |              | unconditional backward |
| `CALL`          | u8 argc     |              | invoke top closure |
| `CLOSURE`       | u8 fn, u8 n |              | create closure over n upvalues |
| `CLOSE_UPVALUE` |             | `v -> ...`   | hoist captured local to heap |
| `RETURN`        |             | `v -> ...`   | return from frame |

Reserved for v2 (declared in enum, never emitted by v1 compiler): `INHERIT`, `METHOD`, `INVOKE`, `SUPER_INVOKE`, `GET_PROPERTY`, `SET_PROPERTY`, `CLASS`.

## 7. VM Model

- **Stack-based.** Operand stack + call-frame stack.
- **CallFrame:** `closure: Rc<Closure>`, `ip: usize`, `slots_offset: usize`.
- **Dispatch:** single `loop { let op = chunk.code[ip]; match OpCode::from(op) { ... } }`.
- **Closures:** compiler walks enclosing function's locals; for each captured local, emit `GET_UPVALUE` / `SET_UPVALUE` instead of `GET_LOCAL` / `SET_LOCAL`. Upvalues have two states: **open** (still on a stack slot) or **closed** (moved to heap on `CLOSE_UPVALUE`).

## 8. Error Handling

- `CompileError` and `RuntimeError` both carry `line: usize` and a `message: String`.
- Compile errors halt at parse/compile; no codegen runs.
- Runtime errors propagate up the call frames; VM prints a stack trace with function names and line numbers.
- No panics in normal operation. `unwrap()` is forbidden on user input paths.

## 9. Testing Strategy

Three layers, all in `cargo test`:

1. **Lexer unit tests** — token streams for tricky inputs (strings, comments, multi-char operators, line tracking, errors).
2. **Parser unit tests** — pretty-printed AST, operator precedence, error recovery.
3. **End-to-end parity tests** — for every `.lang` file in `tests/cases/`, run it through both the interpreter and the VM, assert stdout is identical. This is the killer test.

`tests/cases/stress.lang` covers: deep recursion, many closures, large numbers, all operators, all control flow, error cases.

## 10. CLI / UX

```
$ cargo run -- repl
> let x = 1 + 2;
> print x;
3
> :disassemble
== fn <script> ==
0000 CONST        0    'x'
0002 POP
0003 CONST        1    1
0005 CONST        2    2
0007 ADD
0008 POP
> :quit

$ cargo run -- run examples/fib.lang
55

$ cargo run -- compile examples/fib.lang   # prints disassembly to stdout
```

## 11. Documentation

- `README.md` — what it is, how to build, 30-second demo, the two-executors story.
- `docs/architecture.md` — pipeline diagram, module map, why each piece exists.
- `docs/bytecode.md` — opcode reference, stack effects per op, a worked example disassembly.
- `examples/*.lang` — readable, commented programs.
- `docs/known-limitations.md` — `Rc` cycle caveat, v1 feature list, what's next.

Voice: senior engineer, opinionated, no emoji in technical docs, no marketing tone. Public-portfolio-grade.

## 12. Known Tradeoffs (documented, not hidden)

- **Rc over tracing GC.** Cycles leak. Mitigated by language design. Future `Gc<T>` newtype swaps in cleanly.
- **No classes / inheritance.** v2.
- **No optimization passes.** Single-pass compiler. Documented.
- **No stdlib beyond `print` and `clock`.** Documented.
- **No debug protocol / breakpoints.** Documented.

## 13. Out of Scope (explicit list)

To prevent scope creep:

- Classes, inheritance, `this`, `super`
- Tracing garbage collector
- Optimization (constant folding, peephole, register allocation)
- Standard library beyond `print`, `clock`
- Source maps / debug info beyond line numbers
- JIT, AOT, native code generation
- Modules / imports / package manager
- FFI, native interop
- Async / coroutines
- Tail-call optimization

## 14. Acceptance Criteria

The project is **done** when:

1. `cargo test` is green. All three test layers pass.
2. The parity test suite (`tests/cases/*.lang`) has at least 20 files covering every language feature and every opcode. Both executors produce identical stdout on every file.
3. `cargo run -- repl` works interactively; `:disassemble` produces correct output for at least 5 demo programs.
4. `cargo run -- run examples/fib.lang` and at least 3 other example programs print the expected output.
5. The README, `docs/architecture.md`, and `docs/bytecode.md` exist and are non-trivial.
6. No `unwrap()` on user input paths. No panics reachable from `.lang` source.
7. `cargo clippy -- -D warnings` is clean.
8. `cargo build --release` produces a binary that runs all examples.
