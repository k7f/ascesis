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

If a name given to a structure is `Main`, then it can't be
instantiated explicitly.  Instead, the instantiation is performed when
a `.ces` file containing it is interpreted.

Any _full rule_ may be transformed into an equivalent rule expression
consisting of _open rules_ separated with (infix) addition operator.
The arrow above is thus equivalent to

```rust
ces Arrow { { a -> b } + { b <- a } }
ces Main { Arrow() }
```

Syntactic concatenation of rule expressions, unless '+'-separated, is
interpreted as multiplication of corresponding polynomials.  Next is
the same arrow as above (for brevity, defined directly in `Main`),

```rust
ces Main { { a -> b } { b <- a } }
```

Indeed, in this case we get `b` &bullet; _&theta;_ for effect
polynomial of node `a` and _&theta;_ &bullet; `a` for cause polynomial
of node `b`.

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

A _full rule_ consists of two or more polynomials.  For example, a
rule with four single-node polynomials results in three arrows,

```rust
ces ThreeArrowsInARow(w: Node, x: Node, y: Node, z: Node) { w => x => y => z }
```

Atomic rule expressions are rules and structure instantiations. They
are the two constructs allowed in leaves of a rule expression's AST.

```rust
// seven arrows in a row
ces Main { ThreeArrowsInARow(a, b, c, d) + { d => e } + ThreeArrowsInARow(e, f, g, h) }
```

### Fork

A fork structure may be defined with a _full rule_ (an atomic rule
expression),

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

Like a fork, a choice structure may be defined with a single _full
rule_,

```rust
ces Main { a => b + c } // equivalently, b <= a => c
```

## License

The specification of `cesar` is licensed under the Creative Commons
Attribution 4.0 license.  This implementation of `cesar` is licensed
under the MIT license.  Please read the [LICENSE-CC](LICENSE-CC) and
[LICENSE-MIT](LICENSE-MIT) files in this repository for more
information.
