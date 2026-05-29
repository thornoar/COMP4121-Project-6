use std::io::{self, BufRead};

use rac_diagnostics::{Report, Span, Stage};

use crate::Value;

macro_rules! error {
    ($span:expr, $msg:expr) => {
        Err(Report {
            stage: Stage::Interpreting,
            range: $span,
            msg: $msg,
        })
    };
}

pub fn debug(
    args: Vec<Value>,
    range: Span
) -> Result<Value, Report> {
    if args.len() != 1 {
        return error!(range, format!("The standard function `debug` takes `1` argument, but was supplied `{}`.", args.len()))
    }

    println!("{}", args[0]);
    Ok(Value::Unit)
}

pub fn print_int(
    args: Vec<Value>,
    range: Span
) -> Result<Value, Report> {
    if args.len() != 1 {
        return error!(range, format!("The standard function `printInt` takes `1` argument, but was supplied `{}`.", args.len()))
    }

    match &args[0] {
        Value::Int(val) => {
            println!("{val}");
            Ok(Value::Unit)
        },
        val => error!(range, format!("The standard function `printInt` takes an `integer` argument, but was provided `{}`.", val))
    }
}

pub fn print_string(
    args: Vec<Value>,
    range: Span
) -> Result<Value, Report> {
    if args.len() != 1 {
        return error!(range, format!("The standard function `printString` takes `1` argument, but was supplied `{}`.", args.len()))
    }

    match &args[0] {
        Value::String(val) => {
            println!("{val}");
            Ok(Value::Unit)
        },
        val => error!(range, format!("The standard function `printString` takes an `string` argument, but was provided `{}`.", val))
    }
}

pub fn read_int(
    args: Vec<Value>,
    range: Span
) -> Result<Value, Report> {
    if args.len() != 0 {
        return error!(range, format!("The standard function `readInt` takes `0` arguments, but was supplied `{}`.", args.len()))
    }
    let mut res = String::new();
    match io::stdin().lock().read_line(&mut res) {
        Ok(_) => {
            res.truncate(res.len() - 1);
            match res.parse::<i32>() {
                Ok(val) => Ok(Value::Int(val)),
                Err(_) => error!(range, "Could not parse the input string as an integer.".to_owned())
            }
        }
        Err(_) => error!(range, "Could not read string.".to_owned())
    }
}

pub fn read_string(
    args: Vec<Value>,
    range: Span
) -> Result<Value, Report> {
    if args.len() != 0 {
        return error!(range, format!("The standard function `readString` takes `0` arguments, but was supplied `{}`.", args.len()))
    }
    let mut res = String::new();
    match io::stdin().lock().read_line(&mut res) {
        Ok(_) => {
            res.truncate(res.len() - 1);
            Ok(Value::String(res))
        },
        Err(_) => error!(range, "Could not read string.".to_owned())
    }
}
