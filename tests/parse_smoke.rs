use scripting_vm::ast::{Expr, Stmt};
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::token::Literal;

#[test]
fn parses_full_program() {
    let src = r#"
        var x = 10;
        fn double(n) { return n * 2; }
        if (x > 0 and x < 100) {
            print double(x);
        }
    "#;
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert_eq!(stmts.len(), 3);
    assert!(matches!(stmts[0], Stmt::Var { .. }));
    assert!(matches!(stmts[1], Stmt::Fn { .. }));
    assert!(matches!(stmts[2], Stmt::If { .. }));
}
