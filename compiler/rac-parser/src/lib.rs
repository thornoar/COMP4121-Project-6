//! # RAC Parser
//!
//! This library holds the handwritten recursive-descent parser for Amy with error recovery.
//! Lexical analysis is first performed to produce tokens from a source code string, before that
//! stream of tokens is parsed into an abstract syntax tree for further processing.

#![deny(clippy::pedantic)]
#![deny(unsafe_code)]
// #![deny(warnings)]

pub mod token;
pub mod tokeniter;

use std::collections::{VecDeque};
use std::result::Result;

use rac_ast::*;
use rac_diagnostics::{Report, Span, Stage, join};
use crate::tokeniter::TokenIter;
use crate::token::{TokenKind as TK};

macro_rules! select {
    ($src:expr, $span:expr) => {
        $src[$span.start .. $span.end]
    };
}

macro_rules! expect {
    ($ts:expr, $tk:expr, $msg:expr) => {{
        let token = $ts.pop();
        if token.kind != $tk {
            return Err(Report { stage: Stage::Parsing, span: token.range, msg: String::from($msg) });
        }
        token
    }};
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        Err(Report { stage: Stage::Parsing, span: $span, msg: String::from($msg) })
    };
}

// The top-level parsing function
pub fn parse<'a> (src: &'a [u8]) -> Result<NominalModule, Report> {
    let mut ts = TokenIter::new(src, src.len());
    parse_module(src, &mut ts)
}

// Parses a nominal module.
// Example: `object Test error("asdf") end Test`
fn parse_module<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<NominalModule, Report> {
    expect!(ts, TK::KwObject, "A module must start with the keyword `object`.");
    let id1 = expect!(ts, TK::Identifier, "A module must be given a valid identifier name.");
    let name = get_string(src, id1.range)?;
    let defs = parse_many_definitions(src, ts)?;
    let mexpr = match ts.peek().kind {
        TK::KwEnd => Ok(None),
        _ => parse_expr(src, ts).map(|x| Some(x))
    }?;
    expect!(ts, TK::KwEnd, "A module must end with the keyword `end`.");
    let id2 = expect!(ts, TK::Identifier, "A module must end with its name.");

    if select!(src, id2.range) != select!(src, id1.range) {
        return error!(id2.range, "The names at the start and end of a module must match.");
    }

    let next = ts.pop();
    match next.kind {
        TK::Eof => {},
        _ => { return error!(next.range, "Unexpected token after the module definition.") }
    }
    
    Ok(NominalModule { name: name, defs: defs, expr: mexpr })
}

// Parses a sequence of 0 or more definitions.
// Examples: ``, `def f (): Unit := () end f`
fn parse_many_definitions<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<NominalDefinition>, Report> {
    match ts.peek().kind {
        TK::KwDef | TK::KwAbstract | TK::KwCase => {
            let def = parse_definition(src, ts)?;
            let mut rest = parse_many_definitions(src, ts)?;
            rest.push_back(def);
            Ok(rest)
        },
        _ => Ok(VecDeque::new())
    }
}

// Parses an abstract class, case class, or function definition.
// Examples: `abstract class T`, `case class C(x: Unit) extends T`, `def f(x: String): Int(32) := 5 end f`
fn parse_definition<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<NominalDefinition, Report> {
    let kw = ts.pop();
    match kw.kind {
        TK::KwDef => {
            let id1 = expect!(ts, TK::Identifier, "A function must have a valid name identifier.");
            let name = get_string(src, id1.range)?;
            let args = parse_arglist(src, ts)?;
            expect!(ts, TK::Colon, "Expected a colon after the function argument list.");
            let rt = parse_type(src, ts)?;
            expect!(ts, TK::ColonEqual, "Expected `:=` after the function return type.");
            // parse the function body...
            let body = parse_expr(src, ts)?;
            expect!(ts, TK::KwEnd, "A function body must be followed by the `end` keyword.");
            let id2 = expect!(ts, TK::Identifier, "A function definition must have its name after the `end` keyword.");
            if select!(src, id2.range) != select!(src, id1.range) {
                return error!(id2.range, "The names at the start and end of a function definition must match.");
            }
            Ok(NominalDefinition::FunDef(name, args, rt, body, id1.range))
        },
        TK::KwAbstract => {
            expect!(ts, TK::KwClass, "Expected the keyword `class` after `abstract`.");
            let id = expect!(ts, TK::Identifier, "An abstract class must have a valid name identifier.");
            let name = get_string(src, id.range)?;
            Ok(NominalDefinition::AbstractDef(name, id.range))
        },
        TK::KwCase => {
            expect!(ts, TK::KwClass, "Expected the keyword `class` after `case` in a definition.");
            let id = expect!(ts, TK::Identifier, "A function must have a valid name identifier.");
            let name = get_string(src, id.range)?;
            let args = parse_arglist(src, ts)?;
            expect!(ts, TK::KwExtends, "A case class definition must end with an `extends` clause.");
            let parent = expect!(ts, TK::Identifier, "Expected a valid name of an abstract class.");
            let pname = get_string(src, parent.range)?;
            Ok(NominalDefinition::CaseClassDef(name, args, pname, id.range))
        },
        _ => error!(kw.range, "A module definition must start with either `def`, `abstract`, or `case`."),
    }
}

// Parses an argument list in a constructor or function definition.
// Examples: `(x: String, y: Int(32), z: Unit)`, `()`
fn parse_arglist<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<ArgList<String, Name>, Report> {
    expect!(ts, TK::OpenParen, "Expected an opening parenthesis to start the argument list.");
    match ts.peek().kind {
        TK::CloseParen => {
            ts.consume();
            Ok(VecDeque::new())
        },
        _ => parse_many_arguments(src, ts)
    }
}

// Parses a *non-empty* argument list *with* the closing parenthesis
// Examples: `x: String, y: Int(32), z: Unit \)`
fn parse_many_arguments<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<ArgList<String, Name>, Report> {
    let arg = parse_argument(src, ts)?;
    let delim = ts.pop();
    match delim.kind {
        TK::Comma => {
            let mut lst = parse_many_arguments(src, ts)?;
            lst.push_back(arg);
            Ok(lst)
        }
        TK::CloseParen => {
            let mut lst = VecDeque::new();
            lst.push_back(arg);
            Ok(lst)
        },
        _ => error!(delim.range, "Expected either a comma separator of a closing parenthesis")
    }
}

// Parses an argument in a definition.
// Examples: `x: String`, `y: Int(32)`
fn parse_argument<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<(String, Type<Name>), Report> {
    let id = expect!(ts, TK::Identifier, "An argument must have a valid name identifier.");
    let name = get_string(src, id.range)?;
    expect!(ts, TK::Colon, "Expected a colon after the argument name.");
    let typ = parse_type(src, ts)?;
    Ok((name, typ))
}

// Parses a type.
// Examples: `String`, `Unit`, `Int(32)`, `UserType`
fn parse_type<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Type<Name>, Report> {
    let typ1 = ts.pop();
    match typ1.kind {
        TK::TypInt => {
            expect!(ts, TK::OpenParen, "The `Int` type must be applied to an integer size value.");
            let size = expect!(ts, TK::LitInt, "Expected an integer literal for the `Int` type.");
            match str::from_utf8(&select!(src, size.range)) {
                Ok("32") => {
                    let cp = expect!(ts, TK::CloseParen, "Expected a closing parenthesis.");
                    Ok(Type::IntType(join(typ1.range, cp.range)))
                },
                _ => error!(size.range, "Only size `32` is supported for integer literals.")
            }
        },
        TK::TypBoolean => Ok(Type::BoolType(typ1.range)),
        TK::TypString => Ok(Type::StringType(typ1.range)),
        TK::TypUnit => Ok(Type::UnitType(typ1.range)),
        TK::Identifier => match ts.peek().kind {
            TK::Dot => {
                let owner = get_string(src, typ1.range)?;
                ts.consume();
                let typ2 = expect!(ts, TK::Identifier, "Expected a valid identifier as part of a qualified name.");
                let name = get_string(src, typ2.range)?;
                Ok(Type::ClassType((Some(owner), name), join(typ1.range, typ2.range)))
            },
            _ => get_string(src, typ1.range).map(|s| Type::ClassType((None, s), typ1.range))
        }

            
        _ => error!(typ1.range, "Expected either a primitive type (`Int`, `Boolean`, `String`, or `Unit`), or an identifier.")
    }
}

// Parses an expression, which is a sequence of one or more atomic expressions.
// Examples: `5; x + f(45); "hahaha"`
fn parse_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let cur = parse_atomic_expr(src, ts)?;
    match ts.peek().kind {
        TK::Semicolon => {
            ts.consume();
            let rest = parse_expr(src, ts)?;
            Ok(Expr::Sequence(Box::new(cur), Box::new(rest)))
        }
        _ => Ok(cur)
    }
}

// Parses an atomic expression (i.e. not a sequence of expressions).
// Examples: `5 match { _ => 4 }`, `5 + 6`
fn parse_atomic_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let t1 = ts.pop();
    match t1.kind {
        TK::KwVal => {
            let var_token = expect!(ts, TK::Identifier, "Expected a variable identifier after `val`.");
            let var_name = get_string(src, var_token.range)?;
            expect!(ts, TK::Colon, "Expected a colon after the variable name.");
            let var_type = parse_type(src, ts)?;
            expect!(ts, TK::Equal, "Expected an equal sign after the variable type");
            let var_expr = parse_expr(src, ts)?;
            let sc = expect!(ts, TK::Semicolon, "Expected a semicolon after a `val` declaration.");
            let body = parse_expr(src, ts)?;
            Ok(Expr::Let((None, var_name), var_type, Box::new(var_expr), Box::new(body), join(t1.range, sc.range)))
        }
        TK::KwIf => {
            expect!(ts, TK::OpenParen, "Expected an opening parenthesis after the `if` keyword");
            let cond = parse_expr(src, ts)?;
            expect!(ts, TK::CloseParen, "Expected a closing parenthesis after the `if` condition");
            expect!(ts, TK::KwThen, "Expected the keyword `then`.");
            let if_branch = parse_expr(src, ts)?;
            expect!(ts, TK::KwElse, "Expected the keyword `else`.");
            let else_branch = parse_expr(src, ts)?;
            expect!(ts, TK::KwEnd, "An `if` statement must terminate with `end if`");
            let eif = expect!(ts, TK::KwIf, "An `if` statement must terminate with `end if`");
            Ok(Expr::Ite(Box::new(cond), Box::new(if_branch), Box::new(else_branch), join(t1.range, eif.range)))
        }
        _ => {
            parse_with_match(src, ts)
        }
    }
}

// Parses an expression which may contain the (dreaded) `match` keyword.
// Examples: `a * b / c match { 5 => 0, _ => 1 } - x % 2`
fn parse_with_match<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let mut res = parse_infix_expr(src, ts, 6)?;
    macro_rules! update {
        ($level:expr, $constr:ident) => {{
            ts.consume();
            let rhs = parse_infix_expr(src, ts, $level)?;
            res = Expr::$constr(Box::new(res), Box::new(rhs));
        }};
    }
    loop {
        match ts.peek().kind {
            TK::KwMatch => {
                ts.consume();
                expect!(ts, TK::OpenCurly, "The `match` keyword must be followes by a curly bracket.");
                let mut cases = VecDeque::new();
                loop {
                    let tk = ts.pop();
                    match tk.kind {
                        TK::CloseCurly => {
                            res = Expr::Match(Box::new(res), cases, tk.range);
                            break;
                        }
                        TK::KwCase => {
                            let case = parse_match_case(src, ts)?;
                            cases.push_back(case);
                        }
                        _ => { return error!(tk.range, "Expected either a `case` keyword or a closing curly bracket"); }
                    }
                }
            }
            TK::PipePipe => update!(5, Or),
            TK::AndAnd => update!(4, And),
            TK::EqualEqual => update!(3, Equals),
            TK::Less => update!(2, LessThan),
            TK::LessEquals => update!(2, LessEquals),
            TK::Plus => update!(1, Plus),
            TK::Minus => update!(1, Minus),
            TK::PlusPlus => update!(1, Concat),
            TK::Star => update!(0, Times),
            TK::Slash => update!(0, Div),
            TK::Percent => update!(0, Mod),
            _ => { break; }
        }
    }
    Ok(res)
}

// Parses a match case *without* the `case` keyword.
// Examples: `C(_, _) => 9`, `_ => 4 + x`
fn parse_match_case<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<(Pattern<Name>, Expr<Name>), Report> {
    let pat = parse_pattern(src, ts)?;
    expect!(ts, TK::RightArrow, "A match pattern must be followed by a right arrow.");
    let expr = parse_expr(src, ts)?;
    Ok((pat, expr))
}

// Parses a match pattern.
// Examples: `_`, `"haha"`, `Mod.C(_, 89, _)`
fn parse_pattern<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Pattern<Name>, Report> {
    let tk = ts.pop();
    match tk.kind {
        TK::Underscore => Ok(Pattern::Wildcard(tk.range)),
        TK::LitTrue => Ok(Pattern::BoolPattern(true, tk.range)),
        TK::LitFalse => Ok(Pattern::BoolPattern(false, tk.range)),
        TK::LitInt => get_int(src, tk.range).map(|v| Pattern::IntPattern(v, tk.range)),
        TK::LitString => {
            let val = get_string(src, Span::new(tk.range.start + 1, tk.range.end - 1))?;
            Ok(Pattern::StringPattern(val, tk.range))
        }
        TK::OpenParen => {
            let cp = ts.peek();
            match cp.kind {
                TK::CloseParen => {
                    ts.consume();
                    Ok(Pattern::UnitPattern(join(tk.range, cp.range)))
                }
                _ => error!(cp.range, "Expected a closing parenthesis to finish the unit literal pattern.")
            }
        }
        TK::Identifier => {
            let (qname, idrange) = get_name(src, ts, tk.range)?;
            let next = ts.peek();
            match next.kind {
                TK::OpenParen => {
                    ts.consume();
                    let argpats = parse_pattern_list(src, ts)?;
                    let cp = expect!(ts, TK::CloseParen, "Expected a closing parenthesis at the end of a pattern list");
                    Ok(Pattern::ClassPattern(qname, argpats, join(idrange, cp.range)))
                }
                _ => match qname.0 {
                    None => Ok(Pattern::IdPattern(qname, idrange)),
                    _ => error!(idrange, "Variable names cannot be quantified.")
                }
            }
        }
        _ => error!(tk.range, format!("Undexpected token of kind {:?}", tk.kind))
    }
}

// Parses a list of patterns *without* the surrounding parentheses.
// Examoles: `_, 34, true, Cons(_, _)`
fn parse_pattern_list<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<Pattern<Name>>, Report> {
    match ts.peek().kind {
        TK::CloseParen => { Ok(VecDeque::new()) },
        _ => parse_many_patterns(src, ts)
    }
}

// Parses a *non-empty* list of patterns, *without* the surrounding parentheses.
// Examples: `_, 45, 23`
fn parse_many_patterns<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<Pattern<Name>>, Report> {
    let pat = parse_pattern(src, ts)?;
    let delim = ts.peek();
    match delim.kind {
        TK::Comma => {
            ts.consume();
            let mut rest = parse_many_patterns(src, ts)?;
            rest.push_back(pat);
            Ok(rest)
        },
        TK::CloseParen => {
            let mut rest = VecDeque::new();
            rest.push_back(pat);
            Ok(rest)
        },
        _ => error!(delim.range, "Expected either a comma separator of a closing parenthesis")
    }
}

// Parses an expression which may contain infix operators at different levels of precedence.
//
// The `level` argument denotes the precedence category of infix operators that
// is currently being parsed.
// If an operator `op` has precedence category `n`, then `level < n` means that
// the function will stop parsing if it sees `op` in the token stream.
// When `level` is 0, parsing is stopped as soon as any infix operation is seen.
//
// The precedence categories are as follows:
// - `||` -- level 6
// - `&&` -- level 5
// - `==` -- level 4
// - `<`, `<=` -- level 3
// - `+`, `-`, `++` -- level 2
// - `*`, `/`, `%`  -- level 1
//
// Examples: `x - 5 / 6 + 8 < 7 || f(3)`
fn parse_infix_expr<'a> (src: &'a [u8], ts: &mut TokenIter, level: u8) -> Result<Expr<Name>, Report> {
    if level <= 0 {
        return parse_unary_expr(src, ts);
    }
    let mut res = parse_infix_expr(src, ts, level - 1)?;
    macro_rules! update {
        ($constr:ident) => {{
            ts.consume();
            let rhs = parse_infix_expr(src, ts, level - 1)?;
            res = Expr::$constr(Box::new(res), Box::new(rhs));
        }};
    }
    loop {
        match ts.peek().kind {
            TK::PipePipe if level == 6 => update!(Or),
            TK::AndAnd if level == 5 => update!(And),
            TK::EqualEqual if level == 4 => update!(Equals),
            TK::Less if level == 3 => update!(LessThan),
            TK::LessEquals if level == 3 => update!(LessEquals),
            TK::Plus if level == 2 => update!(Plus),
            TK::Minus if level == 2 => update!(Minus),
            TK::PlusPlus if level == 2 => update!(Concat),
            TK::Star if level == 1 => update!(Times),
            TK::Slash if level == 1 => update!(Div),
            TK::Percent if level == 1 => update!(Mod),
            _ => { break; }
        }
    }
    Ok(res)
}

// Parses an expression which may contain unary operators.
// Examples: `-x`, `!(1 < 2 && x + y >= 7)`
fn parse_unary_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let op = ts.peek();
    match op.kind {
        TK::Minus => {
            ts.consume();
            let operand = parse_simple_expr(src, ts)?;
            Ok(Expr::Neg(Box::new(operand), op.range))
        }
        TK::Bang => {
            ts.consume();
            let operand = parse_simple_expr(src, ts)?;
            Ok(Expr::Not(Box::new(operand), op.range))
        }
        _ => parse_simple_expr(src, ts)
    }
}

// Parses a simple expression.
// Examples: `f(x, y, z)`, `45`, `"haha"`, `error("bad")`
fn parse_simple_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let tk = ts.pop();
    match tk.kind {
        TK::LitInt => get_int(src, tk.range).map(|v| Expr::IntLiteral(v, tk.range)),
        TK::LitTrue => Ok(Expr::BoolLiteral(true, tk.range)),
        TK::LitFalse => Ok(Expr::BoolLiteral(false, tk.range)),
        TK::LitString => get_string(src, Span::new(tk.range.start + 1, tk.range.end - 1)).map(|s| Expr::StringLiteral(s, tk.range)),
        TK::KwError => {
            expect!(ts, TK::OpenParen, "The `error` keyword must be followed by an open parenthesis");
            let arg = parse_expr(src, ts)?;
            let cp = expect!(ts, TK::CloseParen, "Expected a closed parenthesis here.");
            Ok(Expr::Error(Box::new(arg), join(tk.range, cp.range)))
        }
        TK::Identifier => {
            let (qname, idrange) = get_name(src, ts, tk.range)?;
            let next = ts.peek();
            match next.kind {
                TK::OpenParen => {
                    let args = parse_expr_list(src, ts)?;
                    let cp = expect!(ts, TK::CloseParen, "Expected a closing parenthesis at the end of a pattern list");
                    Ok(Expr::Call(qname, args, join(tk.range, cp.range)))
                }
                _ => match qname.0 {
                    None => Ok(Expr::Variable(qname, idrange)),
                    _ => error!(idrange, "Variable names cannot be quantified.")
                }
            }
        }
        TK::OpenParen => {
            let expr = parse_expr(src, ts)?;
            expect!(ts, TK::CloseParen, "Expected a closing parenthesis here.");
            Ok(expr)
        }
        _ => error!(tk.range, format!("Unexpected token kind for a simple expression: {:?}", tk.kind))
    }
}

// Parses a comma-separated list of expressions.
// Examples: `45, x, Cons(234, Nil())`
fn parse_expr_list<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<Expr<Name>>, Report> {
    match ts.peek().kind {
        TK::CloseParen => { Ok(VecDeque::new()) },
        _ => parse_many_exprs(src, ts)
    }
}

// Parses a *non-empty* comma-separated list of expressions.
// Examples: `45, x, Cons(234, Nil())`
fn parse_many_exprs<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<Expr<Name>>, Report> {
    let cur = parse_expr(src, ts)?;
    let delim = ts.peek();
    match delim.kind {
        TK::Comma => {
            ts.consume();
            let mut rest = parse_many_exprs(src, ts)?;
            rest.push_back(cur);
            Ok(rest)
        },
        TK::CloseParen => {
            let mut rest = VecDeque::new();
            rest.push_back(cur);
            Ok(rest)
        },
        _ => error!(delim.range, "Expected either a comma separator of a closing parenthesis")
    }
}

// Parses a string
fn get_string<'a> (src: &'a [u8], span: Span) -> Result<String, Report> {
    match str::from_utf8(&src[span.start .. span.end]) {
        Ok(s) => Ok(String::from(s)),
        _ => error!(span, "This string does not contain valid ASCII.")
    }
}

// Parses an integer literal
fn get_int<'a> (src: &'a [u8], span: Span) -> Result<i32, Report> {
    match str::from_utf8(&src[span.start .. span.end]) {
        Ok(s) => match s.parse::<i32>() {
            Ok(val) => Ok(val),
            _ => error!(span, "Could not parse this as an integer")
        }
        _ => error!(span, "This string does not contain valid ASCII.")
    }
}

// Parses a qualified name, and return its span on success.
fn get_name<'a> (src: &'a [u8], ts: &mut TokenIter, span: Span) -> Result<(Name, Span), Report> {
    match ts.peek().kind {
        TK::Dot => {
            ts.consume();
            let tk2 = expect!(ts, TK::Identifier, "Expected a valid identifier as part of a qualified name.");
            let owner = get_string(src, span)?;
            let name = get_string(src, tk2.range)?;
            Ok(((Some(owner), name), join(span, tk2.range)))
        }
        _ => {
            let name = get_string(src, span)?;
            Ok(((None, name), span))
        }
    }
}
