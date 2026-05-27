use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use rac_diagnostics::{MID, Span, join};

// pub type ArgList<A, T> = VecDeque<(A, T, Span)>;

// Nominal AST structure

#[derive(Debug, Clone)]
pub struct Name {
    pub owner: Option<String>,
    pub name: String,
}

impl Name {
    pub fn new(owner: Option<String>, name: String) -> Self {
        Self { owner, name }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.owner {
            None => write!(f, "{}", self.name),
            Some(owner) => write!(f, "{}.{}", owner, self.name),
        }
    }
}

#[derive(Debug)]
pub enum NominalType {
    IntType(Span),
    BoolType(Span),
    StringType(Span),
    UnitType(Span),
    IdType(Name, Span)
}

pub type NomArgList = VecDeque<(String, NominalType)>;

impl Display for NominalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NominalType::*;
        match self {
            IntType(_) => write!(f, "Int(32)"),
            BoolType(_) => write!(f, "Boolean"),
            StringType(_) => write!(f, "String"),
            UnitType(_) => write!(f, "Unit"),
            IdType(name, _) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug)]
pub struct NominalAbstDef {
    pub name: String,
    pub type_vars: VecDeque<String>,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalClassDef {
    pub name: String,
    pub args: NomArgList,
    pub parent: String,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalFunDef {
    pub name: String,
    pub type_vars: VecDeque<String>,
    pub args: NomArgList,
    pub rt: NominalType,
    pub body: Expr<Name, NominalType>,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalModule {
    pub name: String,
    pub id: MID,
    pub abstract_defs: VecDeque<NominalAbstDef>,
    pub class_defs: VecDeque<NominalClassDef>,
    pub fun_defs: VecDeque<NominalFunDef>,
    pub expr: Option<Expr<Name, NominalType>>,
}

impl NominalModule {
    pub fn print(&self) {
        println!("object {}", self.name);
        for def in self.abstract_defs.iter() {
            println!("   abstract class {}\n", def.name);
        }
        for def in self.class_defs.iter() {
            print!("   case class {} ", def.name);
            let args_str = def
                .args
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect::<Vec<String>>()
                .join(", ");
            println!("({}) extends {}\n", args_str, def.parent);
        }
        for def in self.fun_defs.iter() {
            print!("   def {} ", def.name);
            let args_str = def
                .args
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect::<Vec<String>>()
                .join(", ");
            println!("({}): {} :=", args_str, def.rt);
            println!("      {}", def.body.show(2));
            println!("   end {}\n", def.name);
        }
        match &self.expr {
            Some(e) => println!("   {}", e.show(2)),
            None => {}
        }
        println!("end {}", self.name);
    }
}

// Symbolic (resolved) AST structure

#[derive(Debug, Clone)]
pub enum SymbolicType {
    // Primitive types
    IntType,
    BoolType,
    StringType,
    UnitType,
    // User-defined types
    ClassType(Symbol),
    // Type variables
    Var(Symbol),
}

pub type SymArgList = VecDeque<(Symbol, SymbolicType)>;

impl Display for SymbolicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SymbolicType::*;
        match self {
            IntType => write!(f, "Int(32)"),
            BoolType => write!(f, "Boolean"),
            StringType => write!(f, "String"),
            UnitType => write!(f, "Unit"),
            ClassType(name) => write!(f, "{}", name),
            Var(name) => write!(f, "'{}", name),
        }
    }
}

// #[derive(Debug, Clone, Hash, Eq, PartialEq)]
// pub enum SymbolKind {
//     Variable,
//     Function,
//     Constructor,
//     Type,
//     TypeVariable,
//     Field,
// }

pub type SID = u64;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub id: SID,
    // pub kind: SymbolKind,
}

impl Symbol {
    pub fn new(name: &String, id: SID) -> Self {
        Self { name: name.clone(), id }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.name, self.id)
    }
}

pub struct SymbolGenerator {
    next_id: SID
}

impl SymbolGenerator {
    pub fn new() -> Self {
        SymbolGenerator { next_id: 0 }
    }

    pub fn fresh(&mut self, name: String) -> Symbol {
        let sym = Symbol { name, id: self.next_id };
        self.next_id += 1;
        sym
    }

    pub fn fresh_type_var(&mut self) -> SymbolicType {
        SymbolicType::Var(self.fresh(String::from(format!("'a:{}", self.next_id))))
    }
}

#[derive(Debug)]
pub struct SymbolicAbstDef {
    pub name: Symbol,
    pub type_vars: VecDeque<Symbol>,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicClassDef {
    pub name: Symbol,
    pub args: SymArgList,
    pub parent: Symbol,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicFunDef {
    pub name: Symbol,
    pub type_vars: VecDeque<Symbol>,
    pub args: SymArgList,
    pub rt: SymbolicType,
    pub body: Expr<Symbol, SymbolicType>,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicProgram {
    pub type_defs: HashMap<SID, SymbolicAbstDef>,
    pub class_defs: HashMap<SID, SymbolicClassDef>,
    pub fun_defs: HashMap<SID, SymbolicFunDef>,
    pub exprs: VecDeque<Expr<Symbol, SymbolicType>>,
}

// Expression structures, shared between nominal and symbolic trees.
// The `N` type parameter denotes the "name" type, either `Name` or `Symbol`.
// Some of the constructors include a `Span`, others do not, as the span of an expression
// can sometimes be computed from its subexpressions.

#[derive(Debug, Clone)]
pub enum Expr<N, T> {
    // Variables. The `Span` is the range of the variable
    Variable(N, Span),

    // Literals. The `Span` is the range of the literal
    IntLiteral(i32, Span),
    BoolLiteral(bool, Span),
    StringLiteral(String, Span),
    UnitLiteral(Span),

    // Binary operators. Range is computed as the `join` of the ranges of `lhs` and `rhs`
    Plus(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Minus(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Times(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Div(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Mod(Box<Expr<N,T>>, Box<Expr<N,T>>),
    LessThan(Box<Expr<N,T>>, Box<Expr<N,T>>),
    LessEquals(Box<Expr<N,T>>, Box<Expr<N,T>>),
    And(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Or(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Equals(Box<Expr<N,T>>, Box<Expr<N,T>>),
    Concat(Box<Expr<N,T>>, Box<Expr<N,T>>),

    // Unary operators. The `Span` contains the range of *the operator*, not the whole expression
    Not(Box<Expr<N,T>>, Span),
    Neg(Box<Expr<N,T>>, Span),

    // Function/constructor call. The `Span` contains the range of the *entire expression*
    Call(N, VecDeque<Expr<N,T>>, Span),

    // Control flow
    Sequence(Box<Expr<N,T>>, Box<Expr<N,T>>), // range is computed as the `join` of the ranges of `lhs` and `rhs`
    Let(N, T, Box<Expr<N,T>>, Box<Expr<N,T>>, Span), // The `Span` contains the range of the *entire expression*
    Ite(Box<Expr<N,T>>, Box<Expr<N,T>>, Box<Expr<N,T>>, Span), // The `Span` contains the range of the *entire expression*

    // Pattern matching. The `Span` contains the range of the *closing curly bracket*
    Match(Box<Expr<N,T>>, VecDeque<(Pattern<N>, Expr<N,T>)>, Span),

    // Errors. The `Span` contains the range of the *entire expression*
    Error(Box<Expr<N,T>>, Span),
}

// Computes the *true* range of a given expression.
pub fn range<N,T>(e: &Expr<N,T>) -> Span {
    use Expr::*;
    match e {
        Variable(_, s) => *s,
        IntLiteral(_, s) => *s,
        BoolLiteral(_, s) => *s,
        StringLiteral(_, s) => *s,
        UnitLiteral(s) => *s,
        Plus(lhs, rhs) => join(range(lhs), range(rhs)),
        Minus(lhs, rhs) => join(range(lhs), range(rhs)),
        Times(lhs, rhs) => join(range(lhs), range(rhs)),
        Div(lhs, rhs) => join(range(lhs), range(rhs)),
        Mod(lhs, rhs) => join(range(lhs), range(rhs)),
        LessThan(lhs, rhs) => join(range(lhs), range(rhs)),
        LessEquals(lhs, rhs) => join(range(lhs), range(rhs)),
        And(lhs, rhs) => join(range(lhs), range(rhs)),
        Or(lhs, rhs) => join(range(lhs), range(rhs)),
        Equals(lhs, rhs) => join(range(lhs), range(rhs)),
        Concat(lhs, rhs) => join(range(lhs), range(rhs)),
        Not(ep, s) => join(*s, range(ep)),
        Neg(ep, s) => join(*s, range(ep)),
        Call(_, _, s) => *s,
        Sequence(lhs, rhs) => join(range(lhs), range(rhs)),
        Let(_, _, _, _, s) => *s,
        Ite(_, _, _, s) => *s,
        Match(scrut, _, s) => join(range(scrut), *s),
        Error(_, s) => *s,
    }
}

impl<N: Display, T: Display> Expr<N,T> {
    pub fn show(&self, indent: usize) -> String {
        use Expr::*;
        let prefix1 = "   ".repeat(indent);
        let prefix2 = "   ".repeat(indent + 1);
        macro_rules! binop {
            ($lhs:expr, $op:literal, $rhs:expr) => {
                format!("( {} {} {} )", $lhs.show(indent), $op, $rhs.show(indent))
            };
        }
        match self {
            Variable(name, _) => format!("{}", name),
            IntLiteral(v, _) => format!("{}", v),
            BoolLiteral(v, _) => format!("{}", v),
            StringLiteral(v, _) => format!("\"{}\"", v),
            UnitLiteral(_) => format!("()"),
            Plus(lhs, rhs) => binop!(lhs, "+", rhs),
            Minus(lhs, rhs) => binop!(lhs, "-", rhs),
            Times(lhs, rhs) => binop!(lhs, "*", rhs),
            Div(lhs, rhs) => binop!(lhs, "/", rhs),
            Mod(lhs, rhs) => binop!(lhs, "%", rhs),
            LessThan(lhs, rhs) => binop!(lhs, "<", rhs),
            LessEquals(lhs, rhs) => binop!(lhs, "<=", rhs),
            And(lhs, rhs) => binop!(lhs, "&&", rhs),
            Or(lhs, rhs) => binop!(lhs, "||", rhs),
            Equals(lhs, rhs) => binop!(lhs, "==", rhs),
            Concat(lhs, rhs) => binop!(lhs, "++", rhs),
            Not(e, _) => format!("!({})", e.show(indent)),
            Neg(e, _) => format!("-({})", e.show(indent)),
            Call(name, args, _) => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.show(indent))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{}({})", name, args_str)
            }
            Sequence(lhs, rhs) => {
                format!("{};\n{}{}", lhs.show(indent + 1), prefix1, rhs.show(indent))
            }
            Let(name, typ, val, body, _) => format!(
                "let {}: {} = {} in\n{}( {} )",
                name,
                typ,
                val.show(indent + 1),
                prefix1,
                body.show(indent)
            ),
            Ite(cond, thenb, elseb, _) => format!(
                "if ({}) {{\n{}{}\n{}}} else {{\n{}{}\n{}}}",
                cond.show(indent),
                prefix2,
                thenb.show(indent + 1),
                prefix1,
                prefix2,
                elseb.show(indent + 1),
                prefix1
            ),
            Match(scrut, pats, _) => {
                let pats_str = pats
                    .iter()
                    .map(|(pat, expr)| format!("{}{} => {}", prefix2, pat, expr.show(indent + 2)))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "( {} match {{\n{}\n{}}} )",
                    scrut.show(indent),
                    pats_str,
                    prefix1
                )
            }
            Error(arg, _) => format!("error({})", arg.show(indent)),
        }
        // format!("{}{}", prefix1, expr_str)
    }
}

// Pattern structure

#[derive(Debug, Clone)]
pub enum Pattern<N> {
    Wildcard(Span),
    IdPattern(N, Span),
    BoolPattern(bool, Span),
    StringPattern(String, Span),
    IntPattern(i32, Span),
    UnitPattern(Span),
    ClassPattern(N, VecDeque<Pattern<N>>, Span),
}

impl<N: Display> Display for Pattern<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Pattern::*;
        match self {
            Wildcard(_) => write!(f, "_"),
            IdPattern(name, _) => write!(f, "{}", name),
            BoolPattern(val, _) => write!(f, "{}", val),
            StringPattern(str, _) => write!(f, "\"{}\"", str),
            IntPattern(val, _) => write!(f, "{}", val),
            UnitPattern(_) => write!(f, "()"),
            ClassPattern(name, args, _) => {
                let args_str = args
                    .iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{}({})", name, args_str)
            }
        }
    }
}

// Type structure
