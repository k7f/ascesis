use std::{collections::BTreeMap, iter::FromIterator, str::FromStr, error::Error};
use enquote::unquote;
use crate::bnf_parser::SyntaxParser;

type ParserError = lalrpop_util::ParseError<usize, String, &'static str>;

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

/// Returns `phrase` converted to a `String` after removing all
/// substrings delimited with unquoted ";" on the left and the nearest
/// end of line on the right (delimiters themselves are preserved).
// FIXME spurious semis at eof
pub fn without_comments<S: AsRef<str>>(phrase: S) -> String {
    phrase.as_ref().lines().fold(String::new(), |mut res, line| {
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
    /// To be called only in the parser.
    pub(crate) fn from_rule(rule: Rule) -> Self {
        Self { rules: vec![rule] }
    }

    /// To be called only in the parser.
    pub(crate) fn with_more(mut self, mut other: Self) -> Self {
        self.rules.append(&mut other.rules);
        self
    }

    pub fn from_phrase<S: AsRef<str>>(phrase: S) -> Result<Self, ParserError> {
        let phrase = without_comments(phrase);
        let mut errors = Vec::new();

        let mut result = SyntaxParser::new()
            .parse(&mut errors, &phrase)
            .map_err(|err| err.map_token(|t| format!("{}", t)))?;

        // Sort rules and merge these with common LHS.
        // FIXME merging
        let rules = BTreeMap::from_iter(result.rules.into_iter().map(|rule| (rule.lhs, rule.rhs)));
        result.rules = rules.into_iter().map(|(lhs, rhs)| Rule { lhs, rhs }).collect();

        Ok(result)
    }

    pub fn of_ascesis() -> Self {
        macro_rules! FILE_NAME {
            () => {
                "ascesis_grammar.bnf"
            };
        }

        let phrase = include_str!(FILE_NAME!());

        match Self::from_phrase(phrase) {
            Ok(result) => result,
            Err(err) => panic!("Error in file \"{}\": {}.", FILE_NAME!(), err),
        }
    }

    /// Returns all literals (terminal symbols of the language) in an
    /// alphabetically ordered, deduplicated `Vec`.
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

    /// Returns all rules (each being a group of all productions with
    /// a common LHS symbol) in a slice which is alphabetically
    /// ordered by LHS symbols.
    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl FromStr for Syntax {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_phrase(s)
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
                                id + terminals.len()
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
