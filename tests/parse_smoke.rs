use scripting_vm::ast::{Expr, Stmt};
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::token::Literal;

#[test]
fn parses_minimal() {
    let src = "if (x > 0 and x < 100) { print double(x); }";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let res = Parser::new(toks).parse();
    if let Err(ref e) = res { eprintln!("err: {}", e); }
    let stmts = res.unwrap();
    assert_eq!(stmts.len(), 1);
}
