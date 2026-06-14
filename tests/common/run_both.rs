use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use scripting_vm::compiler::Compiler;
use scripting_vm::interpreter::Interpreter;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::vm::VM;

pub fn run_interpreter(path: &Path) -> String {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return format!("INTERPRETER ERROR: read failed: {}", e),
    };
    let toks = match Lexer::new(&src).scan_tokens() {
        Ok(t) => t,
        Err(e) => return format!("INTERPRETER ERROR: lex: {}", e),
    };
    let prog = match Parser::new(toks).parse() {
        Ok(p) => p,
        Err(e) => return format!("INTERPRETER ERROR: parse: {}", e),
    };
    let out = Rc::new(RefCell::new(Vec::new()));
    let interp = Interpreter::with_sink(out.clone());
    if let Err(e) = interp.run(&prog) {
        return format!("INTERPRETER ERROR: {}", e);
    }
    let snapshot: Vec<String> = out.borrow().clone();
    snapshot.join("\n")
}

pub fn run_vm(path: &Path) -> String {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return format!("VM ERROR: read failed: {}", e),
    };
    let toks = match Lexer::new(&src).scan_tokens() {
        Ok(t) => t,
        Err(e) => return format!("VM ERROR: lex: {}", e),
    };
    let prog = match Parser::new(toks).parse() {
        Ok(p) => p,
        Err(e) => return format!("VM ERROR: parse: {}", e),
    };
    let c = Compiler::new();
    let func = match c.compile(&prog) {
        Ok(f) => f,
        Err(e) => return format!("VM ERROR: compile: {}", e),
    };
    let mut vm = VM::new();
    if let Err(e) = vm.run(func) {
        return format!("VM ERROR: {}", e);
    }
    vm.output.join("\n")
}
