use crate::token::{Token, TokenKind};

#[derive(Clone, Debug)]
pub struct TokenIter<'a> {
    src: &'a [u8],
    limit: usize,
    position: usize,
    cache: Option<Token>
}

impl<'a> TokenIter<'a> {
    pub fn new(src: &'a [u8], limit: usize) -> Self {
        Self { src, limit, position: 0, cache: None }
    }

    pub fn pop(&mut self) -> Token {
        match self.cache {
            Some(tok) => {
                self.cache = None;
                tok
            }
            None => {
                let tok = lex_token(self.src, self.limit, self.position);
                self.position = tok.range.end;
                tok
            }
        }
    }

    pub fn peek(&mut self) -> Token {
        match self.cache {
            Some(tok) => tok,
            None => {
                let tok = lex_token(self.src, self.limit, self.position);
                self.cache = Some(tok);
                self.position = tok.range.end;
                tok
            }
        }
    }

    pub fn consume(&mut self) -> () {
        match self.cache {
            Some(_) => {
                self.cache = None;
            },
            None => {
                let tok = lex_token(self.src, self.limit, self.position);
                self.position = tok.range.end;
            }
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = lex_token(self.src, self.limit, self.position);
        self.position = tok.range.end;

        if tok.kind == TokenKind::Eof {
            None
        } else {
            Some(tok)
        }
    }
}

// Produce the next token from the `start` position.
fn lex_token(src: &[u8], limit: usize, start: usize) -> Token {
    use TokenKind::*;

    // Check if we have any characters left
    if start >= limit {
        return Token::new(Eof, start..start);
    }

    let span = |l| {
        return start .. (start + l);
    };
    let has_next: bool = start+1 < limit;

    match src[start] {
        // Skip whitespace
        c if c.is_ascii_whitespace() => lex_token(src, limit, start+1),
        c if is_id_start(c) => {
            let mut end: usize = start+1;
            while end < limit && is_id_continue(src[end]) {
                end += 1;
            }
            let tk: TokenKind = match str::from_utf8(&src[start..end]) {
                // Keywords
                Ok("abstract") => KwAbstract,
                Ok("case") => KwCase,
                Ok("class") => KwClass,
                Ok("def") => KwDef,
                Ok("else") => KwElse,
                Ok("extends") => KwExtends,
                Ok("if") => KwIf,
                Ok("then") => KwThen,
                Ok("match") => KwMatch,
                Ok("object") => KwObject,
                Ok("val") => KwVal,
                Ok("error") => KwError,
                Ok("end") => KwEnd,
                Ok("_") => Underscore,
                // Primitive types
                Ok("String") => TypString,
                Ok("Int") => TypInt,
                Ok("Boolean") => TypBoolean,
                Ok("Unit") => TypUnit,
                // Literals
                Ok("true") => LitTrue,
                Ok("false") => LitFalse,
                // Otherwise, it's an identifier
                Ok(_) => Identifier,
                Err(_) => Unknown
            };
            Token::new(tk, start..end)
        }

        c if c.is_ascii_digit() => {
            let mut end: usize = start+1;
            while end < limit && src[end].is_ascii_digit() {
                end += 1;
            }
            Token::new(LitInt, start .. end)
        }

        b'&' if has_next && src[start+1] == b'&' => Token::new(AndAnd, span(2)),
        b'!' => Token::new(Bang, span(1)),
        b':' => {
            if has_next && src[start+1] == b'=' {
                Token::new(ColonEqual, span(2))
            } else {
                Token::new(Colon, span(1))
            }
        },
        b',' => Token::new(Comma, span(1)),
        b'.' => Token::new(Dot, span(1)),
        b'=' => {
            if has_next && src[start+1] == b'=' {
                Token::new(EqualEqual, span(2))
            } else if has_next && src[start+1] == b'>' {
                Token::new(RightArrow, span(2))
            } else {
                Token::new(Equal, span(1))
            }
        },
        b'<' => {
            if has_next && src[start+1] == b'=' {
                Token::new(LessEquals, span(2))
            } else {
                Token::new(Less, span(1))
            }
        },
        b'-' => Token::new(Minus, span(1)),
        b'[' => Token::new(OpenBracket, span(1)),
        b'{' => Token::new(OpenCurly, span(1)),
        b'(' => Token::new(OpenParen, span(1)),
        b')' => Token::new(CloseParen, span(1)),
        b'}' => Token::new(CloseCurly, span(1)),
        b']' => Token::new(CloseBracket, span(1)),
        b'%' => Token::new(Percent, span(1)),
        b'|' if has_next && src[start+1] == b'|' => Token::new(PipePipe, span(2)),
        b'+' => {
            if has_next && src[start+1] == b'+' {
                Token::new(PlusPlus, span(2))
            } else {
                Token::new(Plus, span(1))
            }
        }
        b';' => Token::new(Semicolon, span(1)),
        b'/' => {
            if has_next && src[start+1] == b'/' {
                let mut end = start + 2;
                while end < limit && src[end] != b'\n' {
                    end += 1;
                }
                lex_token(src, limit, end + 1)
            } else if has_next && src[start+1] == b'*' {
                let mut end = start + 3;
                while end < limit && (src[end-1] != b'*' || src[end] != b'/') {
                    end += 1;
                }
                // if end == limit {
                //     Token::new(UnclosedComment, start .. end)
                // } else {
                //     lex_token(src, limit, end + 1)
                // }
                lex_token(src, limit, end + 1)
            } else {
                Token::new(Slash, span(1))
            }
        }

            
        b'*' => Token::new(Star, span(1)),
        b'_' => Token::new(Underscore, span(1)),
        b'"' => {
            let mut end = start + 1;
            while end < limit && src[end] != b'"' {
                end += 1;
            }
            Token::new(LitString, start .. (end + 1))
        },
        _ => Token::new(Unknown, span(1)),
    }
}

pub fn is_id_start(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

pub fn is_id_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

#[test]
fn test_tokeniter () {
    println!("--- TOKENS ---");
    let src = "
        abstract class Char
        case class MkChar(code: Int(32)) extends Char

        // A comment
        // A comment
        // 
        // A comment

        abstract class CharList

        /* a multiline comment
                hasdhasd
    ".as_bytes();
    let mut ts = TokenIter::new(src, src.len());
    loop {
        let cur = ts.pop();
        match cur.kind {
            TokenKind::Eof => break,
            tk => match str::from_utf8(&src[cur.range.start .. cur.range.end]) {
                Ok(str) => println!("{:?} -- {:?}", tk, str),
                _ => println!("--- Error ---")
            }
        }
    }
    println!("--- END ---");
    // assert_eq!(0,1)
}
