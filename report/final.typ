#import "style.typ": *
#show: assignment-title-rule(
  title: [COMP4121 Project 6 Report: Polymorphic Types],
  header: [Hong Kong University of Science and Technology #h(1fr) 2026],
  sub1: [MAKSIMOVICH, Roman],
  sub2: [`rmaksimovich@connect.ust.hk`],
  ext1: [WU, Yiu Tsz],
  ext2: [`ytwuac@connect.ust.hk`],
  inset: (left: 9pt, bottom: 9pt)
)

// #show: columns.with(2, gutter: 20pt)
#show heading.where(level: 1): set text(size: 16pt)
#show raw.where(lang: "scala").or(raw.where(lang: "error")): it => {
  box(
    it,
    stroke: (top: .7pt, bottom: .7pt),
    inset: (top: 6pt, bottom: 6pt),
    // width: 9.2cm
    width: 100%,
    // breakable: false,
  )
}

= Introduction

In its existing implementation, the Amy language lacks two prominent features of functional languages: lambda abstractions and parametric polymorphism. We have chosen to implement the latter, i.e. to support
- polymorphic types of the form `T[A_1, ... A_n]`, where the type variables `A_k` may be instantiated to concrete types and constructors extending `T` may depend on `A_k` in the types of their arguments;
- polymorphic functions of the form `f[A_1, ... A_n] (...)`, whose argument types and return type may depend on `A_1, ... A_n`.

In addition, we have undertaken and completed the extra challenge of rewriting the entire compiler frontend from scratch in Rust, including
+ A naïve implementation of the lexer, without reliance on regular expressions;
+ A recursive-descent implementation of the parser;
+ A custom implementation of the name analyzer;
+ A custom implementation of the type-checker, with support for polymorphism;
+ A custom implementation of the interpreter.

Due to time constraints, we have chosen to skip the code generation stage and only implement the interpreter.

Our projects provides, apart from the extra language features, a state-of-the-art error report system, emitting messages that look like this:
#[
  #show: columns.with(2, gutter: 7pt)
  Input:#v(-6pt)
  ```scala
  1  object Test
  2    abstract class Option[A]
  3 
  4    case class None() extends Option
  5    case class Some(x: A) extends W
  6  end Test
  ```
  #colbreak()
  Output:#v(-6pt)
  #box(
    stroke: (top: .7pt, bottom: .7pt),
    inset: (top: 6pt, bottom: 6pt),
    // width: 9.2cm
    width: 1fr,
    [
      `Error during the `#text(blue, `resolving`)` stage.`\
      `-> `#text(blue, `./Test.amy`)`:5:14`\ #v(4pt)
      #text(blue, `4`)`       case class None() extends Option`\
      #text(blue, `5`)`       case class `#text(red, `!! Some(x: A) extends W !!`)
      #text(blue, `6`)`     end Test`\ #v(4pt)
      ```
      Could not find a type named `W` in the `Test` module.
      ```
    ]
  )
  // ```error
  // Error during the resolving stage.
  // -> ./Test.amy:5:14
  //
  // 4       case class None() extends Option
  // 5       case class !! Some(x: A) extends W !!
  // 6     end Test
  //
  // Could not find a type named `NonExistent` in the `Test` module.
  // ```
]

= Examples

Consider the following module which defines a `List[A]` type:
```scala
object TL
  abstract class List[A]
  case class Nil() extends List
  case class Cons(h: A, t: List[A]) extends List

  def switchType[A,B] (xs: List[A]): List[B] :=
    xs match {
      case Nil() => Nil()
      case Cons(_, _) => error("no way to produce B from A.")
    }
  end switchType

  def length[C] (lst: List[C]): Int(32) :=
    lst match {
      case Nil() => 0
      case Cons(h, t) => 1 + length(t)
    }
  end length

  val xs: List[Int(32)] = Cons(3, Cons(7, Nil()));
  val ys: List[Boolean] = switchType(xs);
  ys
end TL
```
This program type-checks and the type variable `A` in `List[A]` is specified to different types in different scenarios. The following program, on the other hand, will not type-check:
```scala
object Test
  abstract class Triple[A, B, C]
  case class MkTriple(x: A, y: B, z: C) extends Triple

  def getFirst[X, Y, Z] (t: Triple[X, Y, Z]): X :=
    t match {
      case MkTriple(x, y, z) => y
    }
  end getFirst
end Test
```
This is because the variable `y` was given type `Y` which is incompatible with the result type `X`.

= Implementation

== Lexer ans parser

In our implementation, lexing and parsing are combined into a single stage.

An input file is lexed by means of a `TokenIter<'a>` struct, which holds a reference `src: &'a [u8]` to the bytes of the file, and a moving index `position: usize`. The next token is produced by analyzing the array `src` starting from the index `position`, producing a `Token` instance, and then moving `position` forward. This implementation is powerful because it allows for cheap lookahead, which simply consists of indexing into `src`.

Our parser does not directly make use of context-free grammars. Instead, it is implemented using a set of mutually recursive functions, each responsible for parsing a particular language structure.

The public interface consists of the function
```rust
pub fn parse<'a>(src: &'a [u8], ts: &mut TokenIter) -> Result<NominalModule, Report>
```
which receives a pointer to an input file and a token iterator, producing a `NominalModule`.

== Name analysis

Name analysis consists of assigning unique labels to symbols throughout the program, and also checking for scope correctness, definition uniqueness, etc. In our implementation, it is done by the function
```rust
pub fn resolve(
    modules: VecDeque<NominalModule>,
    sg: &mut SymbolGenerator,
) -> Result<SymbolicProgram, Report>
```
which receives a list of `NominalModule`'s and a `SymbolGenerator` (which can provide fresh symbols), producing a `SymbolicProgram`. A noticeable difference from the reference compiler is that a list of nominal modules is always merged into a _single_ symbolic program. This is done for simplicity and convenience, since after resolving module names no longer matter.

== Type-checking
