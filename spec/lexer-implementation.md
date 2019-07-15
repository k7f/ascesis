Remarks about lexer implementation
==================================

Any terminal symbol of the language should be presented to the parser
as a single token recognized by the lexer.  Lexer should accept white
space anywhere between tokens and, if a stream of tokens isn't
separated with white space, then lexer should recognize them
separately, unless a result of token concatenation is also a valid
token &emdash; as a rule, longer token always wins.

## Token categories

_Cesar_ terminals are keywords, identifiers, literals, operators,
separators and delimiters.

  - Keywords are `ces`, `labels`, `caps`, `Node`, `CES`, `Size` and
    `String`.

  - Identifiers are unquoted strings of alphanumeric characters (plus
    underscore) not starting from a digit and different from any of
    the keywords.

  - Literals are nonnegative integers and double-quoted strings.

  - Operators are thin and fat arrows, plus sign and asterisk.

  - Separators are colon and comma.

  - Delimiters are parentheses and curly braces.

## Rules for token recognition

```bnf
keyword = "ces" | "labels" | "caps"
        | "Node" | "CES" | "Size" |  "String" ;

identifier = r"[a-zA-Z_][a-zA-Z0-9_]*" - keyword;

literal = size | string ;

size = r"[0-9]+" ;

string = r#""[^"]*""# ;

operator = "->" | "<-" | "=>" | "<=" | "+" | "*" ;

separator = ":" | "," ;

delimiter = "(" | ")" | "{" | "}" ;
```
