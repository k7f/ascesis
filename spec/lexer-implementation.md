Remarks about lexer implementation
==================================

Any terminal symbol of the language should be presented to the parser
as a single token recognized by the lexer.  Lexer should accept white
space anywhere between tokens.  If two consecutive tokens aren't
separated by white space, then lexer should recognize them separately,
unless a result of token concatenation is also a valid token &mdash;
as a rule, longer token always wins.

## Token categories

_Ascesis_ terminal symbols are keywords, identifiers, literals,
operators, separators, delimiters and modifiers.

  - Keywords are `ces`, `vis`, `cap`, `mul`, `inh`, `Node`, `CES`,
    `Size` and `String`.

  - Identifiers are unquoted strings of alphanumeric characters (plus
    underscore) not starting from a digit and different from any of
    the keywords.

  - Literals are nonnegative integers and double-quoted strings.

  - Operators are thin and fat arrows and plus sign.

  - Separators are colon and comma.

  - Delimiters are parentheses and curly braces.

  - Exclamation mark is the only modifier.

## Rules for token recognition

```bnf
keyword = "ces" | "vis" | "cap" | "mul" | "inh"
        | "Node" | "CES" | "Size" |  "String" ;

identifier = r"[a-zA-Z_][a-zA-Z0-9_]*" - keyword;

literal = size | string ;

size = r"[0-9]+" ;

string = r#""[^"]*""# ;

operator = "->" | "<-" | "=>" | "<=" | "+" ;

separator = ":" | "," ;

delimiter = "(" | ")" | "{" | "}" ;

modifier = "!" ;
```
