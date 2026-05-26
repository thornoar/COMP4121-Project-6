use crate::token::{Token, TokenKind};

#[derive(Clone, Debug)]
pub struct TokenIter<'a> {
    src: &'a [u8],
    limit: usize,
    position: usize,
    cache: Option<Token>,
    tag: u8
}

impl<'a> TokenIter<'a> {
    pub fn new(src: &'a [u8], limit: usize, tag: u8) -> Self {
        Self { src, limit, position: 0, cache: None, tag: tag }
    }

    pub fn pop(&mut self) -> Token {
        match self.cache {
            Some(tok) => {
                self.cache = None;
                tok
            }
            None => {
                let tok = lex_token(self.src, self.limit, self.position, self.tag);
                self.position = tok.range.end;
                tok
            }
        }
    }

    pub fn peek(&mut self) -> Token {
        match self.cache {
            Some(tok) => tok,
            None => {
                let tok = lex_token(self.src, self.limit, self.position, self.tag);
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
                let tok = lex_token(self.src, self.limit, self.position, self.tag);
                self.position = tok.range.end;
            }
        }
    }

    pub fn print(&mut self) -> () {
        loop {
            let cur = self.pop();
            match cur.kind {
                TokenKind::Eof => break,
                tk => match str::from_utf8(&self.src[cur.range.start .. cur.range.end]) {
                    Ok(str) => println!("{:?} -- {:?} ({}-{})", str, tk, cur.range.start, cur.range.end),
                    _ => println!("--- Error ---")
                }
            }
        }
    }
}

// impl<'a> Iterator for TokenIter<'a> {
//     type Item = Token;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let tok = lex_token(self.src, self.limit, self.position);
//         self.position = tok.range.end;
//
//         if tok.kind == TokenKind::Eof {
//             None
//         } else {
//             Some(tok)
//         }
//     }
// }

// Produce the next token from the `start` position.
fn lex_token(src: &[u8], limit: usize, start: usize, tag: u8) -> Token {
    use TokenKind::*;

    // Check if we have any characters left
    if start >= limit {
        return Token::new(Eof, start..start, tag);
    }

    let span = |l| {
        return start .. (start + l);
    };
    let has_next: bool = start+1 < limit;

    macro_rules! token {
        ($tk:expr, $range:expr) => {
            Token::new($tk, $range, tag)
        };
    }

    match src[start] {
        // Skip whitespace
        c if c.is_ascii_whitespace() => lex_token(src, limit, start+1, tag),
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
            token!(tk, start..end)
        }

        c if c.is_ascii_digit() => {
            let mut end: usize = start+1;
            while end < limit && src[end].is_ascii_digit() {
                end += 1;
            }
            token!(LitInt, start .. end)
        }

        b'&' if has_next && src[start+1] == b'&' => token!(AndAnd, span(2)),
        b'!' => token!(Bang, span(1)),
        b':' => {
            if has_next && src[start+1] == b'=' {
                token!(ColonEqual, span(2))
            } else {
                token!(Colon, span(1))
            }
        },
        b',' => token!(Comma, span(1)),
        b'.' => token!(Dot, span(1)),
        b'=' => {
            if has_next && src[start+1] == b'=' {
                token!(EqualEqual, span(2))
            } else if has_next && src[start+1] == b'>' {
                token!(RightArrow, span(2))
            } else {
                token!(Equal, span(1))
            }
        },
        b'<' => {
            if has_next && src[start+1] == b'=' {
                token!(LessEquals, span(2))
            } else {
                token!(Less, span(1))
            }
        },
        b'-' => token!(Minus, span(1)),
        b'[' => token!(OpenBracket, span(1)),
        b'{' => token!(OpenCurly, span(1)),
        b'(' => token!(OpenParen, span(1)),
        b')' => token!(CloseParen, span(1)),
        b'}' => token!(CloseCurly, span(1)),
        b']' => token!(CloseBracket, span(1)),
        b'%' => token!(Percent, span(1)),
        b'|' if has_next && src[start+1] == b'|' => token!(PipePipe, span(2)),
        b'+' => {
            if has_next && src[start+1] == b'+' {
                token!(PlusPlus, span(2))
            } else {
                token!(Plus, span(1))
            }
        }
        b';' => token!(Semicolon, span(1)),
        b'/' => {
            if has_next && src[start+1] == b'/' {
                let mut end = start + 2;
                while end < limit && src[end] != b'\n' {
                    end += 1;
                }
                lex_token(src, limit, end + 1, tag)
            } else if has_next && src[start+1] == b'*' {
                let mut end = start + 3;
                while end < limit && (src[end-1] != b'*' || src[end] != b'/') {
                    end += 1;
                }
                if end == limit {
                    token!(UnclosedComment, start .. end)
                } else {
                    lex_token(src, limit, end + 1, tag)
                }
                // lex_token(src, limit, end + 1)
            } else {
                token!(Slash, span(1))
            }
        }

            
        b'*' => token!(Star, span(1)),
        b'_' => token!(Underscore, span(1)),
        b'"' => {
            let mut end = start + 1;
            while end < limit && src[end] != b'"' {
                end += 1;
            }
            token!(LitString, start .. (end + 1))
        },
        _ => token!(Unknown, span(1)),
    }
}

fn is_id_start(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

fn is_id_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    
    #[test]
    fn test_tokeniter () {
        println!("--- TOKENS ---");
        // let src = "
        //     abstract class Char
        //     case class MkChar(code: Int(32)) extends Char
        //
        //     // A comment
        //     // A comment
        //     // 
        //     // A comment
        //
        //     abstract class CharList
        //
        //     /* a multiline comment
        //             hasdhasd
        // ".as_bytes();
        let src = fs::read("../test-files/Test.amy").unwrap();
        let mut ts = TokenIter::new(src.as_slice(), src.len(), 0);
        ts.print();
        println!("--- END ---");
    }
}
