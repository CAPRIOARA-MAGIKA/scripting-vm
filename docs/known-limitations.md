# Known limitations

These are deliberate v1 tradeoffs, not oversights. Documented so reviewers and future-me know what's intentional.

## Reference counting instead of tracing GC

Values are managed via `Rc<RefCell<Obj>>`. Reference cycles leak memory. The v1 language is structured to make cycles hard to produce: there are no mutable fields on heap objects, only `let` rebindings of names. In practice, the shipped test programs do not leak. A future `Gc<T>` newtype can be swapped in for a tracing collector without changing call sites.

## No classes

`class`, `this`, `super`, and inheritance are reserved keywords but not implemented. The opcode set reserves slots for them (see `src/opcode.rs`). This is a deliberate v2.

## No optimization

The compiler is single-pass. No constant folding, no peephole, no register allocation. This is fine for v1; the language is small enough that performance is not a concern.

## No standard library

`print` (as a statement) and `clock` (as a global native) only. A `String` type and a small stdlib are future work.

## No source maps

Errors include line numbers. They do not include column numbers or snippet excerpts. The line table is one entry per emitted byte, which is enough to attribute runtime errors to a source line.

## No debug protocol

No breakpoints, no stepping.

## 256-slot limit

Locals, constants, and upvalues are all addressed with 1-byte indices. Functions with more than 256 locals or constants will fail to compile (`add_constant` returns an error). Plenty of headroom for a v1; not a real constraint.

## Interpreter doesn't implement function calls

The tree-walking reference interpreter handles everything except `Call` expressions, which it errors on with `"function call not yet implemented for interpreter"`. The VM is the only execution path for programs that use functions. The parity test suite is scoped to features both paths implement.

## Non-local upvalues not supported

The compiler captures upvalues that are locals in the *immediate* enclosing function. Multi-level upvalue chains (where a closure captures another closure's local) are not implemented in v1 — the VM panics with "non-local upvalues not supported in v1" if it encounters one.
