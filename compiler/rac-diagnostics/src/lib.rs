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
    Typechecking
}

#[derive(Debug, Clone)]
pub struct Report {
    pub stage: Stage,
    pub span: Span,
    pub msg: String
}

// pub enum Result<T> {
//     Value(T),
//     Error(
//         Span, // span of the erroneous code
//         String // the error message
//     )
// }
//
// impl<T> Result<T> {
//     pub fn bind<V> (self, f: impl FnOnce(T) -> Result<V>) -> Result<V> {
//         match self {
//             Result::Value(t) => f(t),
//             Result::Error(sp, str) => Result::Error(sp, str)
//         }
//     }
// }
