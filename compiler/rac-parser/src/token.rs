use std::{fmt::Display, ops::Range};
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

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        let str = match self {
            Identifier => "an identifier",
            Semicolon => "a sequence delimiter",
            RightArrow => "the `=>` operator",
            AndAnd | EqualEqual | Slash | Star | LessEquals | Less | Minus | Plus | Percent | PipePipe | PlusPlus | Bang => "an operator",
            OpenCurly => "an opening curly brace",
            OpenParen => "an opening parenthesis",
            OpenBracket => "an opening bracket",
            CloseCurly => "an closing curly brace",
            CloseParen => "an closing parenthesis",
            CloseBracket => "an closing bracket",
            Colon | Comma | Dot => "a separator",
            ColonEqual | Equal => "an assignment operator",
            TypInt | TypUnit | TypString | TypBoolean => "a primitive type",
            LitTrue | LitFalse | LitInt | LitString => "a literal value",
            KwAbstract => "the `abstract` keyword",
            KwCase => "the `case` keyword",
            KwClass => "the `class` keyword",
            KwDef => "the `def` keyword",
            KwExtends => "the `extends` keyword",
            KwIf => "the `if` keyword",
            KwThen => "the `then` keyword",
            KwElse => "the `else` keyword",
            KwMatch => "the `match` keyword",
            KwObject => "the `object` keyword",
            KwVal => "the `val` keyword",
            KwError => "the `error` keyword",
            KwEnd => "the `end` keyword",
            UnclosedComment => "an unclosed multiline comment",
            Underscore => "a wildcard",
            Unknown => "an unknown comment",
            Eof => "the end of file"
        };
        write!(f, "{}", str)
    }
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
