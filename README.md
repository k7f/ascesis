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

See [the specification in EBNF](spec/cesar.ebnf).

## Semantics

For now, see [implementation notes](spec/implementation-notes.md).

## Examples

### Single arrow

The simplest _full rule_ defines a single arrow.  Such a rule,
`a => b`, is in the body of the `Arrow` structure defined below.
An instance of the `Arrow` structure is created in the body of the
`Main` structure definition.

```rust
ces Arrow { a => b }
ces Main { Arrow() }
```

Structure `Main` cannot be instantiated explicitly.  Instead, the
instantiation of `Main` is performed when a `.ces` file containing its
definition is being interpreted.  All structures defined in a file
must have unique names.

Any full rule may be transformed into an equivalent rule expression
consisting of a sequence of _open rules_ separated with (infix)
addition operator (cf [implementation
notes](spec/implementation-notes.md#full-rules)).  The arrow above is
thus equivalent to

```rust
ces Arrow { { a -> b } + { b <- a } }
ces Main { Arrow() }
```

If the addition operator is missing between rule expressions, then
their syntactic concatenation will be interpreted as _multiplication_
of corresponding polynomials.

In case of arrow definition, the result of multiplication of the two
open rules is the same as the result of their addition.  For example,
next is the same arrow as above (for brevity, defined directly in
`Main`),

```rust
ces Main { { a -> b } { b <- a } }
```

Indeed, in this case we get `b` &bullet; _&theta;_ for effect
polynomial of node `a`, and _&theta;_ &bullet; `a` for cause
polynomial of node `b`.

By default, node names are equal to node identifiers and node
capacities are equal to 1.  Therefore, in all previous examples they are
declared implicitly as

```rust
nodes { a: { name: "a", cap: 1 }, b: { name: "b", cap: 1 } }
```

Below is a parameterized definition of a single arrow, which is
instantiated in the context providing explicitly specified node names
and increased capacity of node `a`.

```rust
nodes {
    a: { name: "Source", cap: 3 },
    z: { name: "Sink" },
}

ces Arrow(x: Node, y: Node) { x => y }

ces Main { Arrow(a, z) }
```

### Arrow sequence

A full rule consists of two or more polynomials.  For example, a rule
with four single-node polynomials results in three arrows,

```rust
ces ThreeArrowsInARow(w: Node, x: Node, y: Node, z: Node) { w => x => y => z }
```

An atomic rule expression is a single rule or a structure
instantiation.  These are the two constructs allowed in leaves of an
AST of a rule expression.

```rust
// seven arrows in a row
ces Main { ThreeArrowsInARow(a, b, c, d) + { d => e } + ThreeArrowsInARow(e, f, g, h) }
```

### Fork

A fork structure may be defined with a single full rule,

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
    { { a -> b c } { b, c <- a } }
    { { a -> b c } + { b, c <- a } }
}
```

### Choice

Like a fork, a choice structure may be defined with a single full
rule,

```rust
ces Main { a => b + c } // equivalently, b <= a => c
```

Node identifiers occuring in a rule need not be unique.  Next is a
valid definition of a three-way choice.

```rust
ces Main { b <= a => c <= a => d } // equivalent to a => b + c + d
```

## License

The specification of `cesar` is licensed under the Creative Commons
Attribution 4.0 license.  This implementation of `cesar` is licensed
under the MIT license.  Please read the [LICENSE-CC](LICENSE-CC) and
[LICENSE-MIT](LICENSE-MIT) files in this repository for more
information.
