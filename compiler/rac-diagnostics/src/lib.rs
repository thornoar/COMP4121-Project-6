use std::{fmt::Display};

pub type Source<'a> = &'a [u8];

pub type MID = u8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub tag: MID,
}

impl Span {
    pub fn new(start: usize, end: usize, tag: MID) -> Self {
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
            Self::Resolving => write!(f, "resolving"),
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
    eprintln!("Error during the \x1b[34m{}\x1b[0m stage.", r.stage);

    macro_rules! prefix {
        ($line:expr) => {
            format!("\x1b[34m{:<4}\x1b[0m  ", $line)
        };
    }

    let limit = src.len();

    let mut newlines: Vec<usize> = vec![0];

    let beg_nl_idx;
    let end_nl_idx;

    let mut line = 1;
    let mut col = 1;
    let mut curpos = 0;

    while curpos < r.range.start {
        if src[curpos] == b'\n' {
            newlines.push(curpos + 1);
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        curpos += 1;
    }

    beg_nl_idx = newlines.len() - 1;
    while curpos + 1 < r.range.end {
        if src[curpos] == b'\n' {
            newlines.push(curpos + 1);
        }
        curpos += 1;
    }
    end_nl_idx = newlines.len() - 1;

    let mut cnt = 0;
    while curpos + 1 < limit && cnt < 2 {
        if src[curpos] == b'\n' {
            newlines.push(curpos + 1);
            cnt += 1;
        }
        curpos += 1;
    }

    let len = newlines.len();

    eprintln!("-> \x1b[34m{}\x1b[0m:{}:{}\n", fname, line, col);

    if beg_nl_idx > 0 {
        eprint!(
            "{}{}",
            prefix!(line - 1),
            str::from_utf8(&src[newlines[beg_nl_idx - 1]..newlines[beg_nl_idx]])
                .unwrap_or("")
        );
    }
    eprint!(
        "{}{}",
        prefix!(line),
        str::from_utf8(&src[newlines[beg_nl_idx]..r.range.start]).unwrap_or("")
    );

    eprint!("\x1b[31m!! ");
    if beg_nl_idx < end_nl_idx {
        eprint!(
            "{}",
            str::from_utf8(&src[r.range.start..newlines[beg_nl_idx + 1]]).unwrap_or("")
        );
        line += 1;
        let mut nl_idx = beg_nl_idx + 1;
        while nl_idx < end_nl_idx {
            eprint!(
                "{}{}",
                prefix!(line),
                str::from_utf8(&src[newlines[nl_idx]..newlines[nl_idx + 1]]).unwrap_or("")
            );
            nl_idx += 1;
            line += 1;
        }
        eprint!(
            "{}",
            str::from_utf8(&src[newlines[end_nl_idx]..r.range.end]).unwrap_or("")
        );
    } else {
        // println!("hi");
        eprint!(
            "{}",
            str::from_utf8(&src[r.range.start..r.range.end]).unwrap_or("")
        );
    }
    eprint!(" !!\x1b[0m");

    if end_nl_idx < len - 1 {
        eprint!(
            "{}",
            str::from_utf8(&src[r.range.end..newlines[end_nl_idx + 1]]).unwrap_or("")
        );
        line += 1;
        if end_nl_idx < len - 2 {
            eprint!(
                "{}{}",
                prefix!(line),
                str::from_utf8(&src[newlines[beg_nl_idx + 1]..newlines[beg_nl_idx + 2]])
                    .unwrap_or("")
            );
        } else {
            eprint!(
                "{}{}",
                prefix!(line),
                str::from_utf8(&src[newlines[beg_nl_idx + 1]..limit])
                    .unwrap_or("")
            );
        }
    } else {
        eprint!("\n");
    }

    eprintln!("\n{}", r.msg);
}
