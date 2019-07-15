Implementation notes for _cesar_ parser
=======================================

## _Thin_ rules

In principle, a _thin_ structural rule doesn't unfold to a proper
(coherent) c-e structure, except in special cases, where there is a
loop on every non-isolated node.  Thin rules come in four variations.

### _Effect-only_ rules

```ebnf
e_rule = node_list "->" polynomial ;
```

This structural rule specifies an effect `polynomial` for all nodes in
the `node_list`.  An effect-only rule generates

  - set _T_ of sending ports, one port for each node in the
    `node_list`,

  - set _R_ of receiving ports, one port for each node occurring in
    the `polynomial`,

  - set of thin, effect-only links, one for each pair in _T_ &times;
    _R_.

For example, `a, b -> c d` generates ports `[a >]`, `[b >]`, `[c <]`,
`[d <]`, and effect-only links `(a >? c)`, `(a >? d)`, `(b >? c)`, `(b
>? d)`.  Cause polynomials of nodes `c` and `d` are _&theta;_, hence
this structural rule doesn't unfold to a proper c-e structure.

### _Cause-only_ rules

```ebnf
c_rule = node_list "<-" polynomial ;
```

This structural rule specifies a cause `polynomial` for all nodes in
the `node_list`.  A cause-only rule generates

  - set _R_ of receiving ports, one port for each node in the
    `node_list`,

  - set _T_ of sending ports, one port for each node occurring in the
    `polynomial`,

  - set of thin, cause-only links, one for each pair in _T_ &times;
    _R_.

### _Cause-then-effect_ and _forward_ rules

```ebnf
ce_rule = node_list "<-" polynomial "->" polynomial ;
fw_rule = polynomial "->" node_list "->" polynomial ;
```

A cause-then-effect (or forward) rule generates

  - set _T_ of sending ports and a set _R_ of corresponding receiving
    antiports, one sending port and one receiving port for each node
    in the `node_list`,

  - set _T'_ of sending ports, one port for each node occurring in the
    left `polynomial`,

  - set _R'_ of receivingg port, one port for each node occurring in
    the right `polynomial`,

  - set of effect-only links, one for each pair in (_T_ &times; _R'_) \\
    (_T'_ &times; _R_),

  - set of cause-only links, one for each pair in (_T'_ &times; _R_)
    \\ (_T_ &times; _R'_),

  - set of fat links, one for each pair in (_T_ &times; _R'_) &cap;
    (_T'_ &times; _R_).

For example, `a -> b -> a` generates ports `[a >]`, `[a <]`, `[b >]`,
`[b <]`, cause-only link `(a ?> b)`, and effect-only link `(b >? a)`.
This structural rule doesn't unfold to a proper c-e structure, because
both polynomials of node `a` are _&theta;_.

However, since `a b -> a, b -> a b` generates the same ports as above
and four fat links, `(a > a)`, `(b > b)`, `(a > b)`, `(b > a)`, it
unfolds to a proper c-e structure.

### _Effect-then-cause_ and _backward_ rules

```ebnf
ec_rule = node_list "->" polynomial "<-" polynomial ;
bw_rule = polynomial "<-" node_list "<-" polynomial ;
```

These are semantically equivalent to cause-then-effect rules with left
and right polynomials exchanged.  See above.

## _Fat_ rules

```ebnf
fat_rule = polynomial ( "=>" | "<=" ) polynomial { ( "=>" | "<=" ) polynomial } ;
```

A fat structural rule is transformed into a sum ('+'-separated
sequence) of thin rules.  A fat rule with more than two polynomials is
first transformed into a sum of two-polynomial fat rules, for example
`b <= a => c` becomes `{ a => b } + { a => c }`.  Then each
two-polynomial fat rule is replaced with a sum of two thin rules, one
effect-only, another cause-only.  Next, the resulting rule expression
is simplified by integrating effect-only rules having a common node
list and doing the same with cause-only rules.  Finally, rule
expression is further simplified by merging node lists which point to
the same effect polynomials, and merging node lists pointed to by the
same cause polynomials.

For example, `a b c => d e f` is transformed to

```rust
{ a, b, c -> d e f } + { d, e, f <- a b c }
```

`b <= a => c` is transformed to

```rust
{ a -> b + c } + { b <- a } + { c <- a }
```

etc.  A fat rule always unfolds to a proper (coherent) c-e structure.
However, there are structures undefinable with fat rules only, as a
simple triangle structure shows:

```rust
{ a -> b c } + { b <- a c } + { c <- a -> b }
```

## Optional pre-processing step

The syntax defined in the first draft of the surface language as

```ebnf
fw_rule = polynomial "->" node_list "->" polynomial ;
bw_rule = polynomial "<-" node_list "<-" polynomial ;
```

was later replaced with

```ebnf
fw_rule = "+" plain_polynomial "->" node_list "->" polynomial ;
bw_rule = "+" plain_polynomial "<-" node_list "<-" polynomial ;
```

This change was necessary to make the grammar of _cesar_ LR-parsable.
Nevertheless, support for the surface language might still be possible
by pre-processing the input and prefixing some or all of the plain
polynomials with the addition operator.  This would probably involve
right-to-left scanning of the input string, custom lexer
implementation, etc.

### Do we need thin forward rules?

Probably not.  The formula `cause -> state -> effect` might have some
iconic value as a hint to the flow of time (a left-to-right timeline),
but it complicates the implementation and, probably, mental parsing.
