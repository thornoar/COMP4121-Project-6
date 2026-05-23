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
use rac_diagnostics::{Report, Stage, Span, select};
use crate::tokeniter::TokenIter;
use crate::token::TokenKind as TK;

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
    ($span:expr, $msg:literal) => {
        Err(Report { stage: Stage::Parsing, span: $span, msg: String::from($msg) })
    };
}

fn mkstring<'a> (src: &'a [u8], span: Span) -> Result<String, Report> {
    match str::from_utf8(&src[span.start .. span.end]) {
        Ok(s) => Ok(String::from(s)),
        _ => error!(span, "This string does not contain valid ASCII.")
    }
}

type Name = String;

pub fn parse<'a> (src: &'a [u8]) -> Result<Module<Name>, Report> {
    let mut ts = TokenIter::new(src, src.len());
    parse_module(src, &mut ts)
}

// Parses a single Amy module
fn parse_module<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Module<Name>, Report> {
    expect!(ts, TK::KwObject, "A module must start with the keyword `object`.");
    let id1 = expect!(ts, TK::Identifier, "A module must be given a valid identifier name.");
    let name = mkstring(src, id1.range)?;
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
    
    Ok(Module { name: name, defs: defs, expr: mexpr })
}

// Parses a sequence of 0 or more definitions
fn parse_many_definitions<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<VecDeque<Definition<Name>>, Report> {
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

// Parses an abstract class, case class, or function definition
fn parse_definition<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Definition<Name>, Report> {
    let kw = ts.pop();
    match kw.kind {
        TK::KwDef => {
            let id1 = expect!(ts, TK::Identifier, "A function must have a valid name identifier.");
            let name = mkstring(src, id1.range)?;
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
            Ok(Definition::FunDef(name, args, rt, body))
        },
        TK::KwAbstract => {
            expect!(ts, TK::KwClass, "Expected the keyword `class` after `abstract`.");
            let id = expect!(ts, TK::Identifier, "An abstract class must have a valid name identifier.");
            let name = mkstring(src, id.range)?;
            Ok(Definition::AbstractDef(name))
        },
        TK::KwCase => {
            expect!(ts, TK::KwClass, "Expected the keyword `class` after `case` in a definition.");
            let id = expect!(ts, TK::Identifier, "A function must have a valid name identifier.");
            let name = mkstring(src, id.range)?;
            let args = parse_arglist(src, ts)?;
            expect!(ts, TK::KwExtends, "A case class definition must be ended with an `extends` clause.");
            let parent = expect!(ts, TK::Identifier, "Expected a valid name of an abstract class.");
            let pname = mkstring(src, parent.range)?;
            Ok(Definition::CaseClassDef(name, args, pname))
        },
        _ => error!(kw.range, "A definition must start with either `def`, `abstract`, or `case`."),
    }
}

// Parses `(x: String, y: Int(32), z: Unit)` or `()`
fn parse_arglist<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<ArgList<Name>, Report> {
    expect!(ts, TK::OpenParen, "Expected an opening parenthesis to start the argument list.");
    match ts.peek().kind {
        TK::CloseParen => {
            ts.consume();
            Ok(VecDeque::new())
        },
        _ => parse_many_arguments(src, ts)
    }
}

// Parses a *non-empty* argument list `x: String, y: Int(32), z: Unit \)` <- with the closing parenthesis
fn parse_many_arguments<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<ArgList<Name>, Report> {
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

// Parses `x: String` or `y: Int(32)`
fn parse_argument<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<(Name, Type<Name>), Report> {
    let id = expect!(ts, TK::Identifier, "An argument must have a valid name identifier.");
    let name = mkstring(src, id.range)?;
    expect!(ts, TK::Colon, "Expected a colon after the argument name.");
    let typ = parse_type(src, ts)?;
    Ok((name, typ))
}

// Parses `String` or `Unit` or `Int(32)`
fn parse_type<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Type<Name>, Report> {
    let typ = ts.pop();
    match typ.kind {
        TK::TypInt => {
            expect!(ts, TK::OpenParen, "The `Int` type must be applied to an integer size value.");
            let size = expect!(ts, TK::LitInt, "Expected an integer literal for the `Int` type.");
            match str::from_utf8(&select!(src, size.range)) {
                Ok("32") => {
                    expect!(ts, TK::CloseParen, "Expected a closing parenthesis.");
                    Ok(Type::IntType)
                },
                _ => error!(size.range, "Only size `32` is supported for integer literals.")
            }
        },
        TK::TypBoolean => Ok(Type::BoolType),
        TK::TypString => Ok(Type::StringType),
        TK::TypUnit => Ok(Type::UnitType),
        TK::Identifier => mkstring(src, typ.range).map(|s| Type::ClassType(s)),
        _ => error!(typ.range, "Expected either a primitive type (`Int`, `Boolean`, `String`, or `Unit`), or an identifier.")
    }
}

// Parses an expression, which is a sequence of one or more atomic expressions
fn parse_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    todo!()
}

// Parses an atomic expression (i.e. not a sequence of expressions)
fn parse_atomic_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    let t1 = ts.pop();
    match t1.kind {
        TK::KwVal => {
            let var_token = expect!(ts, TK::Identifier, "Expected a variable identifier after `val`.");
            let var_name = mkstring(src, var_token.range)?;
            expect!(ts, TK::Colon, "Expected a colon after the variable name.");
            let var_type = parse_type(src, ts)?;
            expect!(ts, TK::Equal, "Expected an equal sign after the variable type");
            let var_expr = parse_expr(src, ts)?;
            expect!(ts, TK::Semicolon, "Expected a semicolon after a `val` declaration.");
            let body = parse_expr(src, ts)?;
            Ok(Expr::Let(var_name, var_type, Box::new(var_expr), Box::new(body)))
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
            expect!(ts, TK::KwIf, "An `if` statement must terminate with `end if`");
            Ok(Expr::Ite(Box::new(cond), Box::new(if_branch), Box::new(else_branch)))
        }
        _ => {
            parse_infix_expr(src, ts, 7)
        }
    }
}

// Parses an expression which may contain infix operators at different levels of precedence.
//
// The `level` argument denotes the precedence category of infix operators that still needs to be
// considered.
// If an operator `op` has precedence category `n`, then `level < n` means that
// the function will stop parsing if it sees `op` in the token stream.
// When `level` is 0, parsing is stopped as soon as any infix operation is seen.
//
// The precedence categories are as follows:
// - `match` -- level 7
// - `||` -- level 6
// - `&&` -- level 5
// - `==` -- level 4
// - `<`, `<=` -- level 3
// - `+`, `-`, `++` -- level 2
// - `*`, `/`, `%`  -- level 1
fn parse_infix_expr<'a> (src: &'a [u8], ts: &mut TokenIter, level: u8) -> Result<Expr<Name>, Report> {
    if level <= 0 {
        return parse_unary_expr(src, ts);
    }
    let lhs = parse_infix_expr(src, ts, level - 1)?;
    match ts.peek().kind {
        TK::KwMatch if level >= 7 => {
            todo!()
        }
        TK::PipePipe if level >= 6 => {
            todo!()
        }
        TK::AndAnd if level >= 5 => {
            todo!()
        }
        TK::EqualEqual if level >= 4 => {
            todo!()
        }
        TK::Less | TK::LessEquals if level >= 3 => {
            todo!()
        }
        TK::Plus | TK::Minus | TK::PlusPlus if level >= 2 => {
            todo!()
        }
        TK::Star | TK::Slash | TK::Percent => {
            todo!()
        }
        _ => Ok(lhs)
    }
}

// Parse an expression which may contain unary operators
fn parse_unary_expr<'a> (src: &'a [u8], ts: &mut TokenIter) -> Result<Expr<Name>, Report> {
    todo!()
}
