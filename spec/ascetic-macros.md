The `ascetic!` family of Rust macros
====================================

Invocations of any macro from `ascetic!` family of macros are _clean_:
symbols are strictly separated from the enclosing Rust scope.  All
symbols occurring in a plain `ascetic!` or `ascetic_nodes!` macro
invocation are interpreted as literal `Gnid`s.  All symbols used in
the body of `ascetic_rules!` invocation have to be first declared in
its signature.

There are two forms of a c-e structure definition:

  - plain `ascetic!` macro, without suffix, is the immediate
    definition of a `CES` object;

  - the generic form, `ascetic_rules!`, defines a parameterized `CES`
    template by introducing a new Rust macro, so that each invocation
    of this new macro instantiates a c-e structure based on the
    template.

For instance, an immediate form

```rust
let arrow = ascetic! { x => y };
```

defines the same object as the result of the `arrow!` template
instantiation below:

```rust
ascetic_rules! arrow(source: Node, sink: Node) { source => sink }
let (x, y) = ascetic_nodes![x, y];
let arrow = arrow!(x, y);
```

Macro `ascetic_nodes!` accepts a list of literal `Gnid`s and returns a
tuple of `Gnid` objects, which may then be used for template
instantiation and for accessing state, capacities, etc.

```rust
let mut arrow = ascetic! { x => y };

let (x, y) = ascetic_nodes![x, y];

arrow[y].capacity(3);
arrow[x] = 3;

while arrow.shoot() {
    println!("x = {}, y = {}", arrow[x], arrow[y]);
}
println!("deadlock!");
```
