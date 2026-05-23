use std::collections::{HashMap, VecDeque};
use rac_diagnostics::Span;

// Nominal AST structure

pub type Name = (Option<String>, String);
pub enum NominalDefinition {
    AbstractDef(String, Span),
    CaseClassDef(String, ArgList<String, Name>, String, Span),
    FunDef(String, ArgList<String, Name>, Type<Name>, Expr<Name>, Span),
}
pub struct NominalModule {
    pub name: String,
    pub defs: VecDeque<NominalDefinition>,
    pub expr: Option<Expr<Name>>
}

// Symbolic (resolved) AST structure

enum SymbolKind { Variable, Function, Class, Type }
type SID = u64;
struct Symbol {
    name: String,
    id: SID,
    kind: SymbolKind
}

pub struct SymbolicProgram {
    userTypes: VecDeque<Symbol>,
    classDefs: HashMap<SID, (ArgList<Symbol, Symbol>, Symbol)>,
    funDefs: HashMap<SID, (ArgList<Symbol, Symbol>, Type<Symbol>, Expr<Symbol>)>
}

// Expression structures, shared between nominal and symbolic trees.
// The `N` type parameter denotes the "name" type, either `Name` or `Symbol`.
// Some of the constructors include a `Span`, others do not, as the span of an expression
// can sometimes be computed from its subexpressions.

pub enum Expr<N> {
    // Variables
    Variable(N, Span),

    // Literals
    IntLiteral(i32, Span),
    BoolLiteral(bool, Span),
    StringLiteral(String, Span),
    UnitLiteral(Span),
    
    // Binary operators
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

    // Unary operators
    Not(Box<Expr<N>>, Span),
    Neg(Box<Expr<N>>, Span),

    // Function/constructor call
    Call(N, VecDeque<Box<Expr<N>>>, Span),

    // Control flow
    Sequence(Box<Expr<N>>, Box<Expr<N>>),
    Let(N, Type<N>, Box<Expr<N>>, Box<Expr<N>>, Span),
    Ite(Box<Expr<N>>, Box<Expr<N>>, Box<Expr<N>>, Span),

    // Pattern matching
    Match(Box<Expr<N>>, VecDeque<(Box<Expr<N>>, Box<Expr<N>>)>, Span),

    // Errors
    Error(Box<Expr<N>>, Span),
}

pub type ArgList<A, N> = VecDeque<(A, Type<N>)>;

// Pattern structure

pub enum Pattern<N> {
    Wildcard,
    IdPattern(N),
    BoolPattern(bool),
    StringPattern(String),
    IntPattern(i32),
    UnitPattern,
    ClassPattern(N, VecDeque<Pattern<N>>)
}

// Type structure

pub enum Type<N> {
    // Primitive types
    IntType(Span),
    BoolType(Span),
    StringType(Span),
    UnitType(Span),
    // User-defined types
    ClassType(N, Span),
}
