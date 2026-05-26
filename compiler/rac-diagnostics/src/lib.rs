use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Display,
};

pub type Source<'a> = &'a [u8];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub tag: u8,
}

impl Span {
    pub fn new(start: usize, end: usize, tag: u8) -> Self {
        Span { start, end, tag }
    }
}

pub fn join(s1: Span, s2: Span) -> Span {
    Span {
        start: s1.start,
        end: s2.end,
        tag: s1.tag,
    }
}

// pub type Span = (usize, usize);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Stage {
    Parsing,
    Resolving,
    Typechecking,
    Interpreting,
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing => write!(f, "parsing"),
            Self::Resolving => write!(f, "resolution"),
            Self::Typechecking => write!(f, "type checking"),
            Self::Interpreting => write!(f, "interpreting"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub stage: Stage,
    pub range: Span,
    pub msg: String,
}

pub fn deliver(r: &Report, fname: &str, src: &[u8]) {
    let stage_msg = format!("Error during the \x1b[34m{}\x1b[0m stage:", r.stage);

    let limit = src.len();
    let mut line = 1;
    let mut col = 1;
    let mut curpos = 0;
    while curpos < r.range.start {
        if src[curpos] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        curpos += 1;
    }

    let file_msg = format!("-> \x1b[34m{}\x1b[0m:{}:{}", fname, line, col);

    let offset = 10;
    let slice_before =
        str::from_utf8(&src[max(0, curpos - offset)..curpos]).unwrap_or("...decoding error...");
    let slice_err = str::from_utf8(&src[curpos..r.range.end]).unwrap_or("...decoding error...");
    let slice_after = str::from_utf8(&src[r.range.end..min(limit, r.range.end + offset)])
        .unwrap_or("...decoding error...");
    let src_msg = format!(
        "\x1b[34m{}\x1b[0m    {}\x1b[31m{}\x1b[0m{}",
        line, slice_before, slice_err, slice_after
    );

    eprintln!("{}\n{}\n\n{}\n\n-- {}", stage_msg, file_msg, src_msg, r.msg);
}

// impl<'a> Report<'a> {
//     pub fn print (&self) {
//     }
// }
//
