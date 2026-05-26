use std::ops::Range;
use rac_diagnostics::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    // Identifiers
    Identifier,

    // Delimiters
    AndAnd,
    Bang,
    CloseBracket,
    CloseCurly,
    CloseParen,
    Colon,
    ColonEqual,
    Comma,
    Dot,
    Equal,
    EqualEqual,
    LessEquals,
    Less,
    Minus,
    OpenBracket,
    OpenCurly,
    OpenParen,
    Percent,
    PipePipe,
    Plus,
    PlusPlus,
    Semicolon,
    Slash,
    Star,
    RightArrow,

    // Primitive types
    TypString,
    TypInt,
    TypBoolean,
    TypUnit,

    // Built-in literal values for booleans
    LitTrue,
    LitFalse,

    // Integer literals
    LitInt,
    LitString,

    // Keywords
    KwAbstract,
    KwCase,
    KwClass,
    KwDef,
    KwExtends,
    KwIf,
    KwThen,
    KwElse,
    KwMatch,
    KwObject,
    KwVal,
    KwError,
    KwEnd,

    // Misc
    Unknown,
    UnclosedComment,
    Eof,
    Underscore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Span,
}

impl Token {
    pub fn new(kind: TokenKind, range: Range<usize>, tag: u8) -> Self {
        Self {
            kind,
            range: Span { start: range.start, end: range.end, tag: tag }
        }
    }
}
