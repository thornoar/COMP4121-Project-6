use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use rac_diagnostics::{Span, join};

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
pub struct NominalAbstDef {
    pub name: String,
    pub type_vars: VecDeque<String>,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalClassDef {
    pub name: String,
    pub args: ArgList<String, Name>,
    pub parent: String,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalFunDef {
    pub name: String,
    pub type_vars: VecDeque<String>,
    pub args: ArgList<String, Name>,
    pub rt: (Type<Name>, Span),
    pub body: Expr<Name>,
    pub range: Span,
}

#[derive(Debug)]
pub struct NominalModule {
    pub name: String,
    pub abstract_defs: VecDeque<NominalAbstDef>,
    pub class_defs: VecDeque<NominalClassDef>,
    pub fun_defs: VecDeque<NominalFunDef>,
    pub expr: Option<Expr<Name>>,
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
                .map(|(n, t, _)| format!("{}: {}", n, t))
                .collect::<Vec<String>>()
                .join(", ");
            println!("({}) extends {}\n", args_str, def.parent);
        }
        for def in self.fun_defs.iter() {
            print!("   def {} ", def.name);
            let args_str = def
                .args
                .iter()
                .map(|(n, t, _)| format!("{}: {}", n, t))
                .collect::<Vec<String>>()
                .join(", ");
            println!("({}): {} :=", args_str, def.rt.0);
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

    pub fn fresh_type_var(&mut self) -> Type<Symbol> {
        Type::Var(self.fresh(String::from(format!("'a:{}", self.next_id))))
    }
}

#[derive(Debug)]
pub struct SymbolicAbstDef {
    pub name: Symbol,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicClassDef {
    pub name: Symbol,
    pub args: ArgList<Symbol, Symbol>,
    pub parent: Symbol,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicFunDef {
    pub name: Symbol,
    pub args: ArgList<Symbol, Symbol>,
    pub rt: Type<Symbol>,
    pub body: Expr<Symbol>,
    pub range: Span,
}

#[derive(Debug)]
pub struct SymbolicProgram {
    pub user_types: HashMap<SID, SymbolicAbstDef>,
    pub class_defs: HashMap<SID, SymbolicClassDef>,
    pub fun_defs: HashMap<SID, SymbolicFunDef>,
    pub exprs: VecDeque<Expr<Symbol>>,
}

// Expression structures, shared between nominal and symbolic trees.
// The `N` type parameter denotes the "name" type, either `Name` or `Symbol`.
// Some of the constructors include a `Span`, others do not, as the span of an expression
// can sometimes be computed from its subexpressions.

#[derive(Debug, Clone)]
pub enum Expr<N> {
    // Variables. The `Span` is the range of the variable
    Variable(N, Span),

    // Literals. The `Span` is the range of the literal
    IntLiteral(i32, Span),
    BoolLiteral(bool, Span),
    StringLiteral(String, Span),
    UnitLiteral(Span),

    // Binary operators. Range is computed as the `join` of the ranges of `lhs` and `rhs`
    Plus(Box<Expr<N>>, Box<Expr<N>>),
    Minus(Box<Expr<N>>, Box<Expr<N>>),
    Times(Box<Expr<N>>, Box<Expr<N>>),
    Div(Box<Expr<N>>, Box<Expr<N>>),
    Mod(Box<Expr<N>>, Box<Expr<N>>),
    LessThan(Box<Expr<N>>, Box<Expr<N>>),
    LessEquals(Box<Expr<N>>, Box<Expr<N>>),
    And(Box<Expr<N>>, Box<Expr<N>>),
    Or(Box<Expr<N>>, Box<Expr<N>>),
    Equals(Box<Expr<N>>, Box<Expr<N>>),
    Concat(Box<Expr<N>>, Box<Expr<N>>),

    // Unary operators. The `Span` contains the range of *the operator*, not the whole expression
    Not(Box<Expr<N>>, Span),
    Neg(Box<Expr<N>>, Span),

    // Function/constructor call. The `Span` contains the range of the *entire expression*
    Call(N, VecDeque<Expr<N>>, Span),

    // Control flow
    Sequence(Box<Expr<N>>, Box<Expr<N>>), // range is computed as the `join` of the ranges of `lhs` and `rhs`
    Let(N, Type<N>, Box<Expr<N>>, Box<Expr<N>>, Span), // The `Span` contains the range of the *entire expression*
    Ite(Box<Expr<N>>, Box<Expr<N>>, Box<Expr<N>>, Span), // The `Span` contains the range of the *entire expression*

    // Pattern matching. The `Span` contains the range of the *closing curly bracket*
    Match(Box<Expr<N>>, VecDeque<(Pattern<N>, Expr<N>)>, Span),

    // Errors. The `Span` contains the range of the *entire expression*
    Error(Box<Expr<N>>, Span),
}

pub type ArgList<A, N> = VecDeque<(A, Type<N>, Span)>;

// Computes the *true* range of a given expression.
pub fn range<N>(e: &Expr<N>) -> Span {
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

impl<N: Display> Expr<N> {
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

#[derive(Debug, Clone)]
pub enum Type<N> {
    // Primitive types
    IntType,
    BoolType,
    StringType,
    UnitType,
    // User-defined types
    ClassType(N),
    // Type variables
    Var(N),
}

impl<N: Display> Display for Type<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Type::*;
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
