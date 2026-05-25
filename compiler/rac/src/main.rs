//! # RAC
//!
//! This is the main driver program of the rewritten RAC. It connects the moving pieces in a
//! pipeline to drive compilation of Amy into WebAssembly. The concept of a session is used to
//! store certain global state (per compilation) when necessary.

#![deny(clippy::pedantic)]
#![deny(unsafe_code)]
// #![deny(warnings)]

// use tikv_jemallocator::Jemalloc;
// #[global_allocator]
// static ALLOC: Jemalloc = Jemalloc;

use std::{collections::VecDeque, env, fs};

#[derive(Debug, Eq, PartialEq)]
enum Operation {
    PrintTokens,
    PrintNominal,
    PrintResolved,
    TypeCheck,
    Interpret,
    Help
}

macro_rules! init_error {
    ($msg:expr) => {{
        eprintln!("\x1b[31mError:\x1b[0m {}", $msg);
        return;
    }};
}

pub fn main() {
    use Operation::*;

    let mut moper = None;
    let mut fnames: VecDeque<String> = VecDeque::new();
    for arg in env::args() {
        let len = arg.len();
        if len >= 2 && &arg[0..2] == "--" {
            match &arg[2..len] {
                "tokens" => { moper = Some(PrintTokens) },
                "parse" => { moper = Some(PrintNominal) },
                "resolve" => { moper = Some(PrintResolved) },
                "typecheck" => { moper = Some(TypeCheck) },
                "interpret" => { moper = Some(Interpret) },
                "help" => { moper = Some(Help) }
                _ => {
                    init_error!(format!("unrecognized command-line flag: {}", &arg[2..len]));
                }
            }
        } else { fnames.push_back(arg); }
    }
    
    let oper = match moper {
        None => init_error!("no operation given."),
        Some(o) => o
    };
    
    if oper == Help {
        println!(
            "{}\n\n{}\n\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            "This is the Rust Amy Compiler (which is actually an interpreter)",
            "usage: rac [ --OPTIONS ] [ FILE_1 FILE_2 ... ]",
            "one of these options must be given:",
            "  --tokens      print the tokens generated from all input files",
            "  --parse       print the nominal ASTs after parsing, for each file",
            "  --resolve     print the combined symbolic AST after resolution",
            "  --typecheck   typeckeck the combined symbolic AST and print type errors",
            "  --interpret   interpret the symbolic AST and print the execution result",
            "  --help        print this help message"
        );
    }

    let mut sources: VecDeque<Vec<u8>> = VecDeque::new();
    for fname in fnames.iter() {
        match fs::read(&fname) {
            Ok(contents) => { sources.push_back(contents); },
            Err(_) => init_error!(format!("could not read file `{}`", fname))
        }
    }
}
