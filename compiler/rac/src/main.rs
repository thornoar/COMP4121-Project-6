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

use std::{
    collections::{HashMap, VecDeque},
    env, fs,
};

use rac_diagnostics::{MID, deliver};
use rac_parser::{parse, tokeniter::TokenIter};

#[derive(Debug, Eq, PartialEq)]
enum Operation {
    PrintTokens,
    PrintNominal,
    PrintResolved,
    TypeCheck,
    Interpret,
    Help,
}

macro_rules! init_error {
    ($msg:expr) => {{
        eprintln!("\x1b[31mError:\x1b[0m {}", $msg);
        return;
    }};
}

pub fn main() {
    use Operation::*;

    // Stage 1: Command-line argument parsing

    let mut moper = None;
    let mut fnames: VecDeque<String> = VecDeque::new();
    for arg in env::args().skip(1) {
        let len = arg.len();
        if len >= 2 && &arg[0..2] == "--" {
            match &arg[2..len] {
                "tokens" => moper = Some(PrintTokens),
                "parse" => moper = Some(PrintNominal),
                "resolve" => moper = Some(PrintResolved),
                "typecheck" => moper = Some(TypeCheck),
                "interpret" => moper = Some(Interpret),
                "help" => moper = Some(Help),
                _ => {
                    init_error!(format!("unrecognized command-line flag: {}", &arg[2..len]));
                }
            }
        } else {
            fnames.push_back(arg);
        }
    }

    let oper = match moper {
        None => init_error!("no operation given."),
        Some(o) => o,
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

    // Stage 2: File I/O

    let mut sources = VecDeque::new();
    for fname in fnames.iter() {
        match fs::read(&fname) {
            Ok(contents) => {
                sources.push_back((fname.as_str(), contents));
            }
            Err(_) => init_error!(format!("could not read file `\x1b[31m{}\x1b[0m`", fname)),
        }
    }

    // Stage 3: Parsing

    let mut nominal_trees = VecDeque::new();
    let mut curtag = 0;
    let mut srcmap: HashMap<MID, (&str, &[u8])> = HashMap::new();

    for (fname, source) in sources.iter() {
        let src = source.as_slice();
        let mut ts = TokenIter::new(src, src.len(), curtag);
        if oper == PrintTokens {
            ts.print();
        } else {
            match parse(src, &mut ts) {
                Ok(m) => {
                    nominal_trees.push_back(m);
                }
                Err(r) => {
                    deliver(&r, fname, src);
                    return;
                }
            }
            srcmap.insert(curtag, (*fname, src));
            curtag += 1;
        }
    }

    // Stage 4:

    if oper == PrintNominal {
        for module in nominal_trees.iter() {
            module.print();
            print!("\n");
        }
        return;
    }
}
