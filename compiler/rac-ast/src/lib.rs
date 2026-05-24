use std::{collections::{HashMap, VecDeque}, fmt::Display};
use rac_diagnostics::{Span, join};

// Nominal AST structure

#[derive(Debug, Clone)]
pub struct Name {
    pub owner: Option<String>,
    pub name: String
}

impl Name {
    pub fn new(owner: Option<String>, name: String) -> Self {
        Self { owner, name }
    }
}

// pub type Name = (Option<String>, String);
impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.owner {
            None => write!(f, "{}", self.name),
            Some(owner) => write!(f, "{}.{}", owner, self.name)
        }
    }
}

#[derive(Debug, Clone)]
pub enum NominalDefinition {
    AbstractDef(String, Span),
    CaseClassDef(String, ArgList<String, Name>, String, Span),
    FunDef(String, ArgList<String, Name>, Type<Name>, Expr<Name>, Span),
}

#[derive(Debug, Clone)]
pub struct NominalModule {
    pub name: String,
    pub defs: VecDeque<NominalDefinition>,
    pub expr: Option<Expr<Name>>
}

// Symbolic (resolved) AST structure

#[derive(Debug, Clone)]
enum SymbolKind { Variable, Function, Class, Type }
type SID = u64;

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    id: SID,
    kind: SymbolKind
}

#[derive(Debug, Clone)]
pub struct SymbolicProgram {
    userTypes: VecDeque<Symbol>,
    classDefs: HashMap<SID, (ArgList<Symbol, Symbol>, Symbol)>,
    funDefs: HashMap<SID, (ArgList<Symbol, Symbol>, Type<Symbol>, Expr<Symbol>)>
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

pub type ArgList<A, N> = VecDeque<(A, Type<N>)>;

// Computes the *true* range of a given expression.
pub fn range<N> (e: Expr<N>) -> Span {
    use Expr::*;
    match e {
        Variable(_, s) => s,
        IntLiteral(_, s) => s,
        BoolLiteral(_, s) => s,
        StringLiteral(_, s) => s,
        UnitLiteral(s) => s,
        Plus(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Minus(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Times(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Div(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Mod(lhs, rhs) => join(range(*lhs), range(*rhs)),
        LessThan(lhs, rhs) => join(range(*lhs), range(*rhs)),
        LessEquals(lhs, rhs) => join(range(*lhs), range(*rhs)),
        And(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Or(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Equals(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Concat(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Not(ep, s) => join(s, range(*ep)),
        Neg(ep, s) => join(s, range(*ep)),
        Call(_, _, s) => s,
        Sequence(lhs, rhs) => join(range(*lhs), range(*rhs)),
        Let(_, _, _, _, s) => s,
        Ite(_, _, _, s) => s,
        Match(scrut, _, s) => join(range(*scrut), s),
        Error(_, s) => s
    }
}


impl<N: Display> Display for Expr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;

        fn work<M: Display> (e: &Expr<M>, indent: usize) -> String {
            let prefix = " ".repeat(indent);
            let expr_str = match e {
                Variable(name, _) => format!("{}", name),
                IntLiteral(v, _) => format!("{}", v),
                BoolLiteral(v, _) => format!("{}", v),
                StringLiteral(v, _) => format!("\"{}\"", v),
                UnitLiteral(_) => format!("()"),
                Plus(lhs, rhs) => format!("( {} + {} )", work(lhs, 0), work(rhs, 0)),
                Minus(lhs, rhs) => format!("( {} - {} )", work(lhs, 0), work(rhs, 0)),
                Times(lhs, rhs) => format!("( {} * {} )", work(lhs, 0), work(rhs, 0)),
                Div(lhs, rhs) => format!("( {} / {} )", work(lhs, 0), work(rhs, 0)),
                Mod(lhs, rhs) => format!("( {} % {} )", work(lhs, 0), work(rhs, 0)),
                LessThan(lhs, rhs) => format!("( {} < {} )", work(lhs, 0), work(rhs, 0)),
                LessEquals(lhs, rhs) => format!("( {} <= {} )", work(lhs, 0), work(rhs, 0)),
                And(lhs, rhs) => format!("( {} && {} )", work(lhs, 0), work(rhs, 0)),
                Or(lhs, rhs) => format!("( {} || {} )", work(lhs, 0), work(rhs, 0)),
                Equals(lhs, rhs) => format!("( {} == {} )", work(lhs, 0), work(rhs, 0)),
                Concat(lhs, rhs) => format!("( {} ++ {} )", work(lhs, 0), work(rhs, 0)),
                Not(e, _) => format!("!({})", work(e, 0)),
                Neg(e, _) => format!("-({})", work(e, 0)),
                Call(name, args, _) => {
                    let args_str = args.iter().map(|arg| work(arg, 0)).collect::<Vec<String>>().join(", ");
                    format!("{}({})", name, args_str)
                }
                Sequence(lhs, rhs) => format!("{};\n{}", work(lhs, 0), work(rhs, indent)),
                Let(name, typ, val, body, _) => format!("let {}: {} = {} in ({})", name, typ, *val, *body),
                Ite(cond, thenb, elseb, _) => format!(
                    "if ({}) {{\n{}\n{}}} else {{\n{}\n{}}}",
                    work(cond, 0),
                    work(thenb, indent+2),
                    prefix, work(elseb, indent+2),
                    prefix
                ),
                Match(scrut, pats, _) => todo!(),
                Error(arg, _) => format!("error({})", *arg)
            };
            format!("{}{}", prefix, expr_str)
        }

        write!(f, "{}", work(self, 0))

        // macro_rules! binop {
        //     ($lhs:expr, $rhs:expr) => {
        //         write!()
        //     };
        // }
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
    ClassPattern(N, VecDeque<Pattern<N>>, Span)
}

// Type structure

#[derive(Debug, Clone)]
pub enum Type<N> {
    // Primitive types
    IntType(Span),
    BoolType(Span),
    StringType(Span),
    UnitType(Span),
    // User-defined types
    ClassType(N, Span),
}

impl<N: Display> Display for Type<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Type::*;
        match self {
            IntType(_) => write!(f, "Int(32)"),
            BoolType(_) => write!(f, "Boolean"),
            StringType(_) => write!(f, "String"),
            UnitType(_) => write!(f, "Unit"),
            ClassType(name, _) => write!(f, "{}", name)
        }
    }
}
