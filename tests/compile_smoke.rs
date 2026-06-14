use scripting_vm::ast::Program;
use scripting_vm::compiler::Compiler;
use scripting_vm::lexer::Lexer;
use scripting_vm::opcode::OpCode;
use scripting_vm::parser::Parser;

#[test]
fn compile_simple_print() {
    let src = "print 1 + 2;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog: Program = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    let code = func.chunk.code.clone();
    assert!(code.iter().any(|b| *b == OpCode::Print as u8));
    assert!(code.iter().any(|b| *b == OpCode::Add as u8));
}

#[test]
fn compile_closure_emits_closure_opcode() {
    let src = "fn outer() { var x = 1; fn inner() { return x; } }";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    assert!(func
        .chunk
        .code
        .iter()
        .any(|b| *b == OpCode::Closure as u8));
}
