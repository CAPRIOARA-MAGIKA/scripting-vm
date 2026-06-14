# Bytecode Reference

## Encoding

- Each instruction: 1-byte opcode, then 0-2 operand bytes
- Operands: 1 byte for slots/indices, 2 bytes for jump offsets
- A `Chunk` is `Vec<u8> code` + `Vec<Value> constants` + `Vec<usize> lines` (one line per byte, for errors)

## Opcode table

| Opcode | Operands | Effect |
|--------|----------|--------|
| `CONST` | u8 idx | push `constants[idx]` |
| `NIL` | — | push nil |
| `TRUE` / `FALSE` | — | push bool |
| `POP` | — | pop top |
| `GET_LOCAL` | u8 slot | push local at slot |
| `SET_LOCAL` | u8 slot | pop and assign to local |
| `GET_GLOBAL` | u8 name | push global (name is a string constant) |
| `DEFINE_GLOBAL` | u8 name | pop and define global |
| `SET_GLOBAL` | u8 name | pop and assign global |
| `GET_UPVALUE` | u8 idx | push upvalue |
| `SET_UPVALUE` | u8 idx | pop and assign upvalue |
| `ADD` | — | numeric or string concat |
| `SUB` / `MUL` / `DIV` | — | numeric |
| `NEG` | — | numeric negation |
| `NOT` | — | logical not |
| `EQUAL` | — | `==` (with `NOT` used to derive `!=`, `<=`, `>=`) |
| `GREATER` / `LESS` | — | numeric compare |
| `PRINT` | — | pop, push to output buffer |
| `JUMP` | u16 off | unconditional forward jump |
| `JUMP_IF_FALSE` | u16 off | pop, jump if false (does not pop) |
| `LOOP` | u16 off | unconditional backward jump |
| `CALL` | u8 argc | invoke top |
| `CLOSURE` | u8 idx, n upvalues | wrap function in closure |
| `CLOSE_UPVALUE` | — | hoist captured local to heap |
| `RETURN` | — | return from current frame |

## Worked example

For `print 1 + 2;` the compiled chunk is:

```
CONST 0   ; push 1
CONST 1   ; push 2
ADD       ; pop two, push sum
PRINT     ; pop, push to output
NIL
RETURN    ; script return
```

The constant pool contains `[Number(1), Number(2)]`.

## Call frames

A `CallFrame` carries:
- the closure object
- the runtime upvalue list (from the closure)
- the instruction pointer
- the slot offset on the operand stack

A new frame is pushed on `CALL` and popped on `RETURN`. Locals are addressed relative to the frame's slot offset.
