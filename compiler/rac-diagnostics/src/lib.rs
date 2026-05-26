use std::fmt::Display;

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
    eprintln!("Error during the \x1b[34m{}\x1b[0m stage.", r.stage);

    macro_rules! prefix {
        ($line:expr) => {
            format!("\x1b[34m{:2}\x1b[0m    ", $line)
        };
    }

    let limit = src.len();

    let mut newlines: Vec<usize> = Vec::new();

    let mut beg_nl_idx = 0;
    let mut end_nl_idx = 0;

    let mut line = 1;
    let mut col = 1;
    let mut curpos = 0;
    while curpos < r.range.start {
        if src[curpos] == b'\n' {
            newlines.push(curpos);
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        curpos += 1;
    }
    beg_nl_idx = newlines.len() - 1;
    while curpos < r.range.end {
        if src[curpos] == b'\n' {
            newlines.push(curpos);
        }
        curpos += 1;
    }
    end_nl_idx = newlines.len() - 1;

    let mut cnt = 0;
    while curpos < limit && cnt < 2 {
        if src[curpos] == b'\n' {
            newlines.push(curpos);
            cnt += 1;
        }
        curpos += 1;
    }

    let len = newlines.len();

    eprintln!("-> \x1b[34m{}\x1b[0m:{}:{}\n", fname, line, col);

    if beg_nl_idx > 0 {
        eprintln!(
            "{}{}",
            prefix!(line - 1),
            str::from_utf8(&src[(newlines[beg_nl_idx - 1] + 1)..newlines[beg_nl_idx]])
                .unwrap_or("")
        );
    }
    eprint!(
        "{}{}",
        prefix!(line),
        str::from_utf8(&src[(newlines[beg_nl_idx] + 1)..r.range.start]).unwrap_or("")
    );

    eprint!("\x1b[31m");
    if beg_nl_idx < end_nl_idx {
        eprintln!(
            "{}",
            str::from_utf8(&src[r.range.start..newlines[beg_nl_idx + 1]]).unwrap_or("")
        );
        line += 1;
        let mut nl_idx = beg_nl_idx + 1;
        while nl_idx < end_nl_idx {
            eprintln!(
                "{}{}",
                prefix!(line),
                str::from_utf8(&src[(newlines[nl_idx] + 1)..newlines[nl_idx + 1]]).unwrap_or("")
            );
            nl_idx += 1;
            line += 1;
        }
        eprint!(
            "{}",
            str::from_utf8(&src[(newlines[end_nl_idx] + 1)..r.range.end]).unwrap_or("")
        );
    } else {
        // println!("hi");
        eprint!(
            "{}",
            str::from_utf8(&src[r.range.start..r.range.end]).unwrap_or("")
        );
    }
    eprint!("\x1b[0m");

    if end_nl_idx < len - 1 {
        eprintln!(
            "{}",
            str::from_utf8(&src[r.range.end..newlines[end_nl_idx + 1]]).unwrap_or("")
        );
        line += 1;
    } else {
        eprint!("\n");
    }
    if end_nl_idx < len - 2 {
        eprintln!(
            "{}{}",
            prefix!(line),
            str::from_utf8(&src[(newlines[beg_nl_idx + 1] + 1)..newlines[beg_nl_idx + 2]])
                .unwrap_or("")
        );
    }

    eprintln!("\n{}", r.msg);
}

// impl<'a> Report<'a> {
//     pub fn print (&self) {
//     }
// }
//
