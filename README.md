cesar
=====
[![Latest version](https://img.shields.io/crates/v/cesar-lang.svg)](https://crates.io/crates/cesar-lang)
[![docs](https://docs.rs/cesar-lang/badge.svg)](https://docs.rs/cesar-lang)
![Rust](https://img.shields.io/badge/rust-nightly-brightgreen.svg)
![CC-BY-4.0](https://img.shields.io/badge/license-CC-blue.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

A language for [cause-effect
structural](https://link.springer.com/book/10.1007/978-3-030-20461-7)
algebraic representation of systems.

This is part of the [_aces_](https://github.com/k7f/aces) project.

## Syntax

See [the specification in EBNF](spec/cesar-syntax.ebnf).

## Semantics

For now, see [implementation notes](spec/parser-implementation.md).

## Examples

### Single arrow

The simplest _fat arrow rule_ defines a single arrow, for example, `a
=> b`, as in the body of the `Arrow` structure defined below.  An
instance of the `Arrow` structure is created in the body of the `Main`
structure definition.

```rust
ces Arrow { a => b }
ces Main { Arrow() }
```

Structure `Main` cannot be instantiated explicitly.  Instead, the
instantiation of `Main` is performed when a `.ces` file containing its
definition is being interpreted.  All structure identifiers defined in
a file must be unique.

Any fat arrow rule is equivalent to a rule expression consisting of a
sequence of _thin arrow rules_ separated with (infix) addition
operator.  The fat-into-thin (_FIT_) transformation steps are sketched
out in [implementation
notes](spec/parser-implementation.md#fat-arrow-rules).  The arrow
above is thus equivalent to

```rust
ces Arrow { { a -> b } + { b <- a } }
ces Main { Arrow() }
```

If the addition operator is missing between rule expressions, then
their syntactic concatenation will be interpreted as _multiplication_
of corresponding polynomials.

In case of arrow definition, the result of multiplication of the two
thin arrow rules is the same as the result of their addition.  For
example, next is the same arrow as above (for brevity, defined
directly in `Main`),

```rust
ces Main { { a -> b } { b <- a } }
```

Indeed, in this case we get `b` &bullet; _&theta;_ for effect
polynomial of node `a`, and _&theta;_ &bullet; `a` for cause
polynomial of node `b`.

### Context

By default, node labels are equal to node identifiers, node capacities
are equal to 1, all node-to-monomial multiplicities are equal to 1,
and there are no inhibitors.  Therefore, in all previous examples they
are declared implicitly as

```rust
vis { labels { a: "a", b: "b" } }
cap { 1 a b }
mul { 1 a -> b, 1 b <- a }
```

What follows is a parameterized definition of a single arrow, which is
instantiated in the context providing explicitly specified node labels
and increased capacity of node `a`.

```rust
ces Arrow(x: Node, y: Node) { x => y }

vis { labels { a: "Source", z: "Sink" } }
cap { 3 a }

ces Main { Arrow(a, z) }
```

### Immediate and template definitions

FIXME

### Arrow sequence

A fat arrow rule consists of two or more polynomials.  For example, a
fat arrow rule with four single-node polynomials results in three
arrows,

```rust
ces ThreeArrowsInARow(w: Node, x: Node, y: Node, z: Node) { w => x => y => z }
```

An atomic rule expression is a single arrow rule or a structure
instantiation.  These are the two constructs allowed in leaves of an
AST of a rule expression.

```rust
// seven arrows in a row
ces Main { ThreeArrowsInARow(a, b, c, d) + { d => e } + ThreeArrowsInARow(e, f, g, h) }
```

### Fork

A fork structure may be defined with a single fat arrow rule,

```rust
ces Main { a => b c }
```

Each of the rule expressions below is an alternative definition of the
same fork as defined above.  The final result, their product, is the
same fork as well.

```rust
ces Main {
    { b c <= a }
    { { a => b } { a => c } }
    { { a -> b c } { b c <- a } }
    { { a -> b c } + { b c <- a } }
}
```

### Choice

Like a fork, a choice structure may be defined with a single fat arrow
rule,

```rust
ces Main { a => b + c } // equivalently, b <= a => c
```

Node identifiers occuring in an arrow rule need not be unique.  Next
is a valid definition of a three-way choice,

```rust
ces Main { b <= a => c <= a => d } // equivalent to a => b + c + d
```

and another expression, where the choice is between a set of nodes and
its subset:

```rust
ces Main { a => b c + b } // equivalent to { a => b c } + { a => b }
```

## License

The specification of `cesar` is licensed under the Creative Commons
Attribution 4.0 license.  This implementation of `cesar` is licensed
under the MIT license.  Please read the [LICENSE-CC](LICENSE-CC) and
[LICENSE-MIT](LICENSE-MIT) files in this repository for more
information.
