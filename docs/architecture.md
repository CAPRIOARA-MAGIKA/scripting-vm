# Architecture

## Pipeline

```
source  --lexer-->  tokens  --parser-->  AST
                                       |
                                 +-----+-----+
                                 |           |
                         (interpreter)  (compiler)
                                 |           |
                             values       bytecode
                                            |
                                           VM
                                            |
                                         values
```

## Why two executors

The interpreter is the reference. When the VM's behavior diverges from it on any test, you have a bug. The parity test in `tests/parity.rs` enforces this on every build.

The interpreter covers most of the language but does not implement function calls in v1 (it errors on `Call` expressions). The VM is the full executor. Parity tests are scoped to features both paths implement.

## Module map

| Module | Owns | Depends on |
|--------|------|------------|
| `token` | Token, TokenType | — |
| `lexer` | source -> tokens | `token` |
| `ast` | Expr, Stmt | `token` (for Literal) |
| `error` | CompileError, RuntimeError | — |
| `parser` | tokens -> AST | `lexer`, `ast`, `error` |
| `value` | Value enum | `obj` |
| `obj` | Obj, ObjKind | `value`, `opcode`, `upvalue` |
| `env` | lexical scopes | `value`, `error` |
| `interpreter` | tree-walking eval | `ast`, `env`, `value`, `obj`, `error`, `native` |
| `opcode` | OpCode, Chunk | `value` |
| `upvalue` | Upvalue, UpvalueRef | `value` |
| `compiler` | AST -> bytecode | `ast`, `opcode`, `upvalue`, `value`, `obj`, `error`, `native` |
| `vm` | bytecode execution | `compiler`, `opcode`, `value`, `obj`, `upvalue`, `native`, `error` |
| `native` | built-in functions | `value` |
| `main` | CLI dispatch | all of the above |

The dependency graph is one-way: nothing in `compiler` or `vm` is imported by `interpreter`. Both consume the same AST.

## Closures

When the compiler sees a function declaration inside another function, it walks the enclosing function's locals. Each captured local becomes an **upvalue** on the inner function. At runtime, `CLOSURE` reads the upvalue operands and walks the chain — for a local upvalue it captures the stack slot; for a non-local it walks to the enclosing closure.

Each upvalue is a `Rc<RefCell<Upvalue>>`. Upvalues have two states:
- **open** — the captured local is still on the VM's stack
- **closed** — when the enclosing scope ends, the value is hoisted to the heap so the closure can outlive the stack frame

## Errors

- Compile errors halt at parse or compile time
- Runtime errors propagate up the call stack; the VM prints a stack trace with function names. Per-byte line tracking is wired in but the compiler currently emits `0` for every opcode's line (a v1 simplification); the runtime therefore reports `line 0` for runtime errors. Compile errors do carry correct line numbers. See `docs/known-limitations.md`.
- No `unwrap()` on user input paths
