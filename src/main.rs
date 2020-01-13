#![feature(test)]
#[macro_use]
pub mod common;
pub mod arena;
pub mod ast;
pub mod builtins;
pub mod bytecode;
pub mod cfg;
pub mod compile;
mod display;
pub mod dom;
pub mod harness;
pub mod lexer;
pub mod llvm;
pub mod runtime;
pub mod types;
extern crate elsa;
extern crate hashbrown;
extern crate jemallocator;
extern crate lalrpop_util;
extern crate lazy_static;
extern crate libc;
extern crate llvm_sys;
extern crate petgraph;
extern crate rand;
extern crate regex;
extern crate ryu;
extern crate smallvec;
extern crate stable_deref_trait;
extern crate unicode_xid;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(syntax);

// TODO: put jemalloc behind a feature flag
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const _PROGRAM: &'static str = r#"
function fib(n) {
if (n == 0 || n == 1) {
return n;
}
return fib(n-1) + fib(n-2);
}
END { print fib(35); }"#;

const _PROGRAM_2: &'static str = r#"
END { for (i=0; i<100000000; i++) {SUM += i;}; print SUM }"#;
const _PROGRAM_3: &'static str = r#"
END { for (i=0; i<100000000; i++) {SUMS[i]++; SUM += i;}; print SUM }"#;
const PROGRAM_4: &'static str = r#"
END { for (i=0; i<1000000; i++) {SUMS[i ""]++; SUM += i;}; print SUM }"#;

fn main() {
    unsafe { llvm::test_codegen() };
    // TODO add a real main function
    if false {
        let a = arena::Arena::default();
        println!("{}", harness::run_program(&a, PROGRAM_4, "").unwrap().0);
    }
    eprintln!("exiting cleanly");
}
