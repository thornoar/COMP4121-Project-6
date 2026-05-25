use std::{cmp::{max, min}, fmt::Display};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self { Span { start, end } }
}

pub fn join (s1: Span, s2: Span) -> Span {
    Span { start: s1.start, end: s2.end }
}

// pub type Span = (usize, usize);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Stage {
    Parsing,
    Resolving,
    Typechecking,
    Interpreting
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing => write!(f, "parsing"),
            Self::Resolving => write!(f, "resolution"),
            Self::Typechecking => write!(f, "type checking"),
            Self::Interpreting => write!(f, "interpreting")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Report<'a> {
    pub src: &'a [u8],
    pub stage: Stage,
    pub range: Span,
    pub msg: String
}

impl<'a> Report<'a> {
    pub fn print (&self) {
        let stage_msg = format!("Compilation error during the *{}* stage:", self.stage);
        let limit = self.src.len();
        let mut line = 1;
        let mut col = 1;
        let mut curpos = 0;
        while curpos < self.range.start {
            if self.src[curpos] == b'\n' {
                line += 1; col = 1;
            } else {
                col += 1;
            }
            curpos += 1;
        }
        let offset = 10;
        let slice = str::from_utf8(&self.src[max(0, curpos - offset) .. min(limit, self.range.end + offset)]).unwrap_or("...decoding error...");
        let src_msg = format!("{}:{}    {}", line, col, slice);
        eprintln!("{}\n{}\n-- {}", stage_msg, src_msg, self.msg);
    }
}

