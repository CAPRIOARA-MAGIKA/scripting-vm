use scripting_vm::ast::Program;
use scripting_vm::interpreter::Interpreter;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn arith_and_print() {
    let src = "print 1 + 2 * 3;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog: Program = Parser::new(toks).parse().unwrap();
    let out = Rc::new(RefCell::new(Vec::new()));
    let interp = Interpreter::with_sink(out.clone());
    interp.run(&prog).unwrap();
    assert_eq!(out.borrow().concat(), "7");
}
