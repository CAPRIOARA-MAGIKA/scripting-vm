use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use scripting_vm::compiler::Compiler;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => {
            let path = match args.get(2) {
                Some(p) => p,
                None => {
                    eprintln!("usage: scripting-vm run <file>");
                    std::process::exit(64);
                }
            };
            let src = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {}", path, e);
                    std::process::exit(74);
                }
            };
            run_source(&src);
        }
        Some("repl") | None => repl(),
        Some(other) => {
            eprintln!("unknown subcommand: {}", other);
            eprintln!("usage: scripting-vm [run <file> | repl]");
            std::process::exit(64);
        }
    }
}

fn run_source(src: &str) {
    let toks = match Lexer::new(src).scan_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(65);
        }
    };
    let prog = match Parser::new(toks).parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(65);
        }
    };
    let c = Compiler::new();
    let func = match c.compile(&prog) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(65);
        }
    };
    let mut vm = VM::new();
    if let Err(e) = vm.run(func) {
        eprintln!("{}", e);
        std::process::exit(70);
    }
    for line in vm.output {
        println!("{}", line);
    }
}

fn repl() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    {
        let mut out = stdout.lock();
        let _ = writeln!(out, "scripting-vm REPL. ctrl-c or :quit to exit.");
        let _ = out.flush();
    }
    let mut vm = VM::new();
    loop {
        {
            let mut out = stdout.lock();
            let _ = write!(out, "> ");
            let _ = out.flush();
        }
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {}", e);
                break;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == ":quit" {
            break;
        }
        let toks = match Lexer::new(line).scan_tokens() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let prog = match Parser::new(toks).parse() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let c = Compiler::new();
        let func = match c.compile(&prog) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        if let Err(e) = vm.run(func) {
            eprintln!("{}", e);
            continue;
        }
        for l in &vm.output {
            println!("{}", l);
        }
        vm.output.clear();
    }
}
