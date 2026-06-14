mod common;
use common::run_both::*;
use std::path::Path;

fn check(name: &str) {
    let path = Path::new("tests/cases").join(name);
    let a = run_interpreter(&path);
    let b = run_vm(&path);
    assert_eq!(a, b, "parity mismatch in {}", name);
}

#[test] fn parity_arith()   { check("arith.lang"); }
#[test] fn parity_vars()    { check("vars.lang"); }
#[test] fn parity_print()   { check("print.lang"); }
