mod common;
use common::run_both::*;
use std::path::Path;

fn check(name: &str) {
    let path = Path::new("tests/cases").join(name);
    let a = run_interpreter(&path);
    let b = run_vm(&path);
    if a != b {
        debug_run(name, &path);
    }
    assert_eq!(a, b, "parity mismatch in {}", name);
}

#[test] fn parity_arith()   { check("arith.lang"); }
#[test] fn parity_vars()    { check("vars.lang"); }
#[test] fn parity_print()   { check("print.lang"); }
#[test] fn parity_if()          { check("if.lang"); }
#[test] fn parity_while()       { check("while.lang"); }
#[test] fn parity_logical()     { check("logical.lang"); }
#[test] fn parity_cmp()         { check("cmp.lang"); }
#[test] fn parity_strings()     { check("strings.lang"); }
#[test] fn parity_precedence()  { check("precedence.lang"); }
#[test] fn parity_unary()       { check("unary.lang"); }
#[test] fn parity_group()       { check("group.lang"); }
#[test] fn parity_scope()       { check("scope.lang"); }
#[test] fn parity_global_reassign() { check("global_reassign.lang"); }
#[test] fn parity_stress()      { check("stress.lang"); }
#[test] fn parity_nested_blocks()   { check("nested_blocks.lang"); }
#[test] fn parity_multi_assign()    { check("multi_assign.lang"); }
#[test] fn parity_chained_cmp()     { check("chained_cmp.lang"); }
#[test] fn parity_empty()           { check("empty.lang"); }
#[test] fn parity_mixed_operands()  { check("mixed_operands.lang"); }
#[test] fn parity_nested_loops()    { check("nested_loops.lang"); }
#[test] fn parity_deep_nesting()    { check("deep_nesting.lang"); }
