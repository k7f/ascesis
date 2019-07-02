Implementation notes for _cesar_
================================

## _Open_ rules

In principle, an _open_ cause-effect rule doesn't unfold to a proper
(coherent) c-e structure, except in special cases, where there is a
loop on every non-isolated node.  _Open_ rules come in four
variations.

### _Effect-only_ rules

```ebnf
e_rule = node_list "->" polynomial ;
```

This rule specifies an effect `polynomial` for all nodes in the
`node_list`.  An _effect-only_ rule generates

  - set _T_ of sending ports, one port for each node in the
    `node_list`,

  - set _R_ of receiving ports, one port for each node occurring in
    the `polynomial`,

  - set of effect-only links, one for each pair in _T_ &times; _R_.

For example, `a, b -> c d` generates ports `[a >]`, `[b >]`, `[c <]`,
`[d <]`, and effect-only links `(a > c)`, `(a > d)`, `(b > c)`, `(b >
d)`.  Cause polynomials of nodes `c` and `d` are _&theta;_, therefore
this rule doesn't unfold to a proper c-e structure.

### _Cause-only_ rules

```ebnf
c_rule = node_list "<-" polynomial ;
```

This rule specifies a cause `polynomial` for all nodes in the
`node_list`.  A _cause-only_ rule generates

  - set _R_ of receiving ports, one port for each node in the
    `node_list`,

  - set _T_ of sending ports, one port for each node occurring in the
    `polynomial`,

  - set of cause-only links, one for each pair in _T_ &times; _R_.

### _Cause-then-effect_ rules

```ebnf
ce_rule = polynomial "->" node_list "->" polynomial ;
```

A _cause-then-effect_ rule generates

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

  - set of full links, one for each pair in (_T_ &times; _R'_) &cap;
    (_T'_ &times; _R_).

For example, `a -> b -> a` generates ports `[a >]`, `[a <]`, `[b >]`,
`[b <]`, cause-only link `(a > b)`, and effect-only link `(b > a)`.
Both polynomials of node `a` are _&theta;_, therefore this rule
doesn't unfold to a proper c-e structure.

However, `a b -> a, b -> a b` generates the same ports as above and
four full links, `(a > a)`, `(b > b)`, `(a > b)`, `(b > a)`, therefore
it unfolds to a proper c-e structure.

### _Effect-then-cause_ rules

```ebnf
ec_rule = polynomial "<-" node_list "<-" polynomial ;
```

These are semantically equivalent to _cause-then-effect_ rules with
left and right polynomials exchanged.  See above.

## _Full_ rules

A _full_ rule always unfolds to a proper (coherent) c-e structure.
However, there are structures undefinable with full rules only.
