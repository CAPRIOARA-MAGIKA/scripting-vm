use scripting_vm::compiler::Compiler;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;

#[test]
#[ignore]
fn dump_compiled_closure() {
    let src = r#"
        fn make() {
          var i = 0;
          fn tick() { i = i + 1; return i; }
          return tick;
        }
        var c = make();
        print c();
    "#;
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog = Parser::new(toks).parse().unwrap();
    let mut c = Compiler::new();
    // Touch globals before compile consumes self.
    let _ = &mut c;
    let func = c.compile(&prog).unwrap();
    println!("code bytes: {:?}", func.chunk.code);
    println!("constants count: {}", func.chunk.constants.len());
    for (i, k) in func.chunk.constants.iter().enumerate() {
        println!("  const[{}] = {:?}", i, k);
    }
    println!("\nmake's chunk: {:?}", func.chunk.constants[0]);
}

use scripting_vm::vm::VM;

#[test]
fn vm_runs_arith() {
    let src = "print 1 + 2 * 3;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    let mut vm = VM::new();
    let out = vm.run(func);
    assert!(out.is_ok(), "vm error: {:?}", out.err());
    assert_eq!(vm.output.join("|"), "7");
}

#[test]
fn vm_runs_closure_counter() {
    let src = r#"
        fn make() {
          var i = 0;
          fn tick() { i = i + 1; return i; }
          return tick;
        }
        var c = make();
        print c();
        print c();
        print c();
    "#;
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    let mut vm = VM::new();
    vm.run(func).unwrap();
    assert_eq!(vm.output.join("|"), "1|2|3");
}
