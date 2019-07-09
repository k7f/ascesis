use std::{str::FromStr, error::Error};
use enquote::unquote;
use crate::{ParsingError, ParsingResult, bnf_parser::SyntaxParser};

fn first_unquoted_semi<S: AsRef<str>>(line: S) -> Option<usize> {
    let mut is_quoted = false;
    let mut is_escaped = false;

    for (pos, ch) in line.as_ref().chars().enumerate() {
        if is_quoted {
            if ch == '\\' {
                is_escaped = !is_escaped;
            } else if ch == '"' && !is_escaped {
                is_quoted = false;
            } else {
                is_escaped = false;
            }
        } else if ch == '"' {
            is_quoted = true;
            is_escaped = false;
        } else if ch == ';' {
            return Some(pos)
        }
    }
    None
}

/// Returns `spec` converted to a `String` after removing all
/// substrings delimited with unquoted ";" on the left and the nearest
/// end of line on the right (delimiters themselves are preserved).
pub fn without_comments<S: AsRef<str>>(spec: S) -> String {
    spec.as_ref().lines().fold(String::new(), |mut res, line| {
        if let Some(pos) = first_unquoted_semi(line) {
            res.push_str(&line[..=pos]);
        } else {
            res.push_str(line);
        }
        res.push('\n');
        res
    })
}

#[derive(Debug)]
pub struct Syntax {
    rules: Vec<Rule>,
}

impl Syntax {
    pub(crate) fn from_rule(rule: Rule) -> Self {
        Self { rules: vec![rule] }
    }

    // FIXME merge rules with equal lhs
    pub(crate) fn with_more(mut self, mut other: Self) -> Self {
        self.rules.append(&mut other.rules);
        self
    }

    pub(crate) fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = without_comments(spec);
        let mut errors = Vec::new();

        let result = SyntaxParser::new()
            .parse(&mut errors, &spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }

    pub fn of_cesar() -> Self {
        macro_rules! FILE_NAME {
            () => {
                "cesar_grammar.bnf"
            };
        }

        let spec = include_str!(FILE_NAME!());

        match Self::from_spec(spec) {
            Ok(result) => result,
            Err(err) => panic!("Error in file \"{}\": {}.", FILE_NAME!(), err),
        }
    }

    /// Returns all literal symbols (language terminals) as a sorted, deduplicated `Vec`.
    pub fn get_literals(&self) -> Vec<String> {
        let mut result = Vec::new();

        for rule in self.rules.iter() {
            for list in rule.rhs.lists.iter() {
                for term in list.terms.iter() {
                    if let Term::Literal(lit) = term {
                        result.push(lit.clone());
                    }
                }
            }
        }
        result.sort();
        let len = result.partition_dedup().0.len();
        result.truncate(len);

        result
    }

    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl FromStr for Syntax {
    type Err = ParsingError;

    fn from_str(s: &str) -> ParsingResult<Self> {
        Self::from_spec(s)
    }
}

#[derive(Debug)]
pub struct Rule {
    lhs: String,
    rhs: Expression,
}

impl Rule {
    pub(crate) fn new(lhs: String, rhs: Expression) -> Self {
        Self { lhs, rhs }
    }

    pub fn get_lhs(&self) -> &str {
        &self.lhs
    }

    pub fn get_rhs_list(&self, terminals: &[String], nonterminals: &[String]) -> Vec<Vec<usize>> {
        self.rhs
            .lists
            .iter()
            .map(|list| {
                list.terms
                    .iter()
                    .map(|term| match term {
                        Term::Literal(lit) => {
                            if let Ok(id) = terminals.binary_search(&lit) {
                                id
                            } else {
                                panic!("Unexpected terminal symbol \"{}\" in BNF grammar.", lit)
                            }
                        }
                        Term::RuleName(name) => {
                            if let Ok(id) = nonterminals.binary_search(&name) {
                                id
                            } else {
                                panic!("Undefined nonterminal symbol <{}> in BNF grammar.", name);
                            }
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Expression {
    lists: Vec<List>,
}

impl Expression {
    pub(crate) fn from_list(list: List) -> Self {
        Self { lists: vec![list] }
    }

    pub(crate) fn with_more(mut self, mut other: Self) -> Self {
        self.lists.append(&mut other.lists);
        self
    }
}

#[derive(Debug)]
pub struct List {
    terms: Vec<Term>,
}

impl List {
    pub(crate) fn from_term(term: Term) -> Self {
        Self { terms: vec![term] }
    }

    pub(crate) fn with_more(mut self, mut other: Self) -> Self {
        self.terms.append(&mut other.terms);
        self
    }
}

#[derive(Debug)]
pub enum Term {
    Literal(String),
    RuleName(String),
}

impl Term {
    pub(crate) fn new_literal(quoted: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self::Literal(unquote(&quoted)?))
    }

    pub(crate) fn new_rule_name(name: String) -> Self {
        Self::RuleName(name)
    }
}
