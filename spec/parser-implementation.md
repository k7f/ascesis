Implementation notes for the _Ascesis_ parser
=============================================

## _Thin arrow_ rules

In principle, a _thin arrow_ rule doesn't unfold to a proper
(coherent) c-e structure, except in special cases, where there is a
loop on every non-isolated node.  Thin arrow rules come in four
variations.

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

For example, `a b -> c d` generates ports `[a >]`, `[b >]`, `[c <]`,
`[d <]`, and effect-only links `(a >? c)`, `(a >? d)`, `(b >? c)`, `(b
>? d)`.  Cause polynomials of nodes `c` and `d` are _&theta;_, hence
this rule doesn't unfold to a proper c-e structure.

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
This rule doesn't unfold to a proper c-e structure, because both
polynomials of node `a` are _&theta;_.

However, since `a b -> a b -> a b` generates the same ports as above
and four fat links, `(a > a)`, `(b > b)`, `(a > b)`, `(b > a)`, it
unfolds to a proper c-e structure.

### _Effect-then-cause_ and _backward_ rules

```ebnf
ec_rule = node_list "->" polynomial "<-" polynomial ;
bw_rule = polynomial "<-" node_list "<-" polynomial ;
```

These are semantically equivalent to cause-then-effect rules with left
and right polynomials exchanged.  See above.

## _Fat arrow_ rules

```ebnf
fat_arrow_rule = polynomial ( "=>" | "<=" ) polynomial { ( "=>" | "<=" ) polynomial } ;
```

A fat arrow rule is transformed into a sum ('+'-separated sequence) of
thin arrow rules.  This procedure takes several steps.  A fat arrow
rule with more than two polynomials is first transformed into a sum of
two-polynomial fat arrow rules, for example `b <= a => c` becomes `{ a
=> b } + { a => c }`.  Then each two-polynomial fat arrow rule is
replaced with a sum of two thin arrow rules, one effect-only, another
cause-only.  Next,

  - the resulting rule expression is simplified by integrating
    effect-only rules having a common node list and doing the same
    with cause-only rules; subsequently,

  - rule expression is further simplified by merging node lists which
    point to the same effect polynomials, and merging node lists
    pointed to by the same cause polynomials.

Last two steps are repeated, until a fixed point is reached.  Finally,
since the result is a sum of single-polynomial thin arrow rules, any
pair of rules with the same node list is combined into a
two-polynomial rule.

For example, `a b c => d e f` is transformed to

```rust
{ a b c -> d e f } + { d e f <- a b c }
```

`b <= a => c` is transformed to

```rust
{ a -> b + c } + { b <- a } + { c <- a }
```

etc.  A fat arrow rule always unfolds to a proper (coherent) c-e
structure.  However, there are structures undefinable with fat arrow
rules only, as a simple triangle structure shows:

```rust
{ a -> b c } + { b <- a c } + { c <- a -> b }
```

## Two forms of c-e structure instantiation

Template instantiations are syntactically distinguished from immediate
instantiations, similarly to Rust macro invocations, which differ from
Rust function calls.  When a c-e structure name is used in a template
instantiation, it must be followed by the exclamation mark.  [The
specification](ascesis-syntax.ebnf) therefore defines two productions
for the `ces_instance` nonterminal,

```ebnf
ces_instance = identifier "(" ")"
             | identifier "!" "(" instance_args ")" ;
```

where `instance_args` expects a nonempty list of arguments,

```ebnf
instance_args = arg_value { ","  arg_value } [ "," ] ;
```

The reason for decorating template instantiations with exclamation
mark is twofold.  Without,

  - the standalone _Ascesis_ language would be less consistent with
    [Rust-embedded `ascetic` DSL](ascetic-macros.md), and

  - it would contain LR-unparsable sentences, hence requiring to
    precede proper parsing with a disambiguation pass.

## What is a node list?

A node list is defined in [the specification](ascesis-syntax.ebnf) as
a sequence of node identifiers,

```ebnf
node_list = identifier { identifier } ;
```

However, in the files
[`ascesis_grammar.bnf`](../src/ascesis_grammar.bnf) and
[`ascesis_parser.lalrpop`](../src/ascesis_parser.lalrpop) `NodeList`
is defined as an alias of the `Polynomial` nonterminal.  If, instead,
`NodeList` was implemented as a separate nonterminal with a narrower
sublanguage than that of `Polynomial`, then the current grammar of
_Ascesis_ couldn't be transformed directly into an LR parser.

Therefore, an object of type `Polynomial` carries a flag indicating
whether it is a monomial which was constructed from a syntactically
flat list of node identifiers, and thus it qualifies as a valid node
list following the specification.  Node lists are to be notated
without parentheses or an addition operator &mdash; leading plus sign
is not allowed, nor an expression reducible by idempotence of
addition.

## Do we need thin forward rules?

Probably not, and they may be removed from future versions of the
language, once it is clear that pushing node list from the front of a
thin arrow rule, to the middle, will cause confussion or make _mental_
parsing harder.  On the other hand, they may stay in the language, if
it turns out that the more important factor will be the iconic value
of the formula `cause -> state -> effect` as a hint to the flow of
time, a left-to-right timeline.

## How to support incremental construction?

The idea is to be able to declare template arguments as `Hybrid`, so
that they would accept passing nodes as well as structures.
