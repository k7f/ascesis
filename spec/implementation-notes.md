Implementation notes for _cesar_
================================

## _Thin_ rules

In principle, a _thin_ cause-effect rule doesn't unfold to a proper
(coherent) c-e structure, except in special cases, where there is a
loop on every non-isolated node.  Thin rules come in four variations.

### _Effect-only_ rules

```ebnf
e_rule = node_list "->" polynomial ;
```

This rule specifies an effect `polynomial` for all nodes in the
`node_list`.  An effect-only rule generates

  - set _T_ of sending ports, one port for each node in the
    `node_list`,

  - set _R_ of receiving ports, one port for each node occurring in
    the `polynomial`,

  - set of thin, effect-only links, one for each pair in _T_ &times;
    _R_.

For example, `a, b -> c d` generates ports `[a >]`, `[b >]`, `[c <]`,
`[d <]`, and effect-only links `(a >? c)`, `(a >? d)`, `(b >? c)`,
`(b >? d)`.  Cause polynomials of nodes `c` and `d` are _&theta;_,
hence this rule doesn't unfold to a proper c-e structure.

### _Cause-only_ rules

```ebnf
c_rule = node_list "<-" polynomial ;
```

This rule specifies a cause `polynomial` for all nodes in the
`node_list`.  A cause-only rule generates

  - set _R_ of receiving ports, one port for each node in the
    `node_list`,

  - set _T_ of sending ports, one port for each node occurring in the
    `polynomial`,

  - set of thin, cause-only links, one for each pair in _T_ &times;
    _R_.

### _Cause-then-effect_ rules

```ebnf
ce_rule = polynomial "->" node_list "->" polynomial ;
```

A cause-then-effect rule generates

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
This rule doesn't unfold to a proper c-e structure, because both
polynomials of node `a` are _&theta;_.

However, since `a b -> a, b -> a b` generates the same ports as above
and four fat links, `(a > a)`, `(b > b)`, `(a > b)`, `(b > a)`, it
unfolds to a proper c-e structure.

### _Effect-then-cause_ rules

```ebnf
ec_rule = polynomial "<-" node_list "<-" polynomial ;
```

These are semantically equivalent to cause-then-effect rules with left
and right polynomials exchanged.  See above.

## _Fat_ rules

```ebnf
fat_rule = polynomial ( "=>" | "<=" ) polynomial { ( "=>" | "<=" ) polynomial } ;
```

A fat rule is transformed into a sum ('+'-separated sequence) of thin
rules.  A fat rule with more than two polynomials is first transformed
into a sum of two-polynomial fat rules, for example `b <= a => c`
becomes `{ a => b } + { a => c }`.  Then each two-polynomial rule is
replaced with a sum of two thin rules, one effect-only, another
cause-only.  Next, the resulting rule expression is simplified by
integrating effect-only rules having a common node list and doing the
same with cause-only rules.  Finally, rule expression is further
simplified by merging node lists which point to the same effect
polynomials, and merging node lists pointed to by the same cause
polynomials.

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
{ a -> b c } + { b <- a c } + { a -> c -> b }
```
