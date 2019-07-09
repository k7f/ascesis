#![feature(slice_partition_dedup)]

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(
    #[allow(clippy::all)]
    pub cesar_parser
);

lalrpop_mod!(
    #[allow(clippy::all)]
    pub bnf_parser
);

pub mod bnf;
pub mod grammar;
pub mod sentence;

use std::{fmt, collections::BTreeSet, iter::FromIterator, str::FromStr, error::Error};
use enquote::unquote;
use crate::cesar_parser::RexParser;

pub type ParsingError = lalrpop_util::ParseError<usize, String, String>;
pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Clone, Debug)]
pub enum CesarError {
    ParsingError(String),
}

impl Error for CesarError {
    fn description(&self) -> &str {
        "cesar error"
    }
}

impl fmt::Display for CesarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<ParsingError> for CesarError {
    fn from(_err: ParsingError) -> Self {
        CesarError::ParsingError("parsing error".to_owned())
    }
}

#[derive(Debug)]
pub struct Rex {
    rule: Rule,
}

impl Rex {
    pub(crate) fn from_rule(rule: Rule) -> Self {
        Rex { rule }
    }

    pub(crate) fn with_more(self, _rexlist: Vec<(Option<BinOp>, Rex)>) -> Self {
        self
    }

    pub(crate) fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = RexParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

impl FromStr for Rex {
    type Err = ParsingError;

    fn from_str(s: &str) -> ParsingResult<Self> {
        Self::from_spec(s)
    }
}

#[derive(Debug)]
pub enum Rule {
    Thin(ThinRule),
    Fat(FatRule),
}

#[derive(Default, Debug)]
pub struct ThinRule {
    nodes:  NodeList,
    cause:  Polynomial,
    effect: Polynomial,
}

impl ThinRule {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_nodes(mut self, nodes: NodeList) -> Self {
        self.nodes = nodes;
        self
    }

    pub(crate) fn with_cause(mut self, cause: Polynomial) -> Self {
        self.cause = cause;
        self
    }

    pub(crate) fn with_effect(mut self, effect: Polynomial) -> Self {
        self.effect = effect;
        self
    }
}

#[derive(Default, Debug)]
pub struct FatRule {
    causes:  Vec<Polynomial>,
    effects: Vec<Polynomial>,
}

impl FatRule {
    pub(crate) fn from_parts(head: Polynomial, tail: Vec<(BinOp, Polynomial)>) -> Self {
        assert!(!tail.is_empty(), "Single-polynomial fat rule");

        let mut rule = Self::default();
        let mut prev = head;

        for (op, poly) in tail.into_iter() {
            match op {
                BinOp::FatTx => {
                    rule.causes.push(prev);
                    rule.effects.push(poly.clone());
                }
                BinOp::FatRx => {
                    rule.effects.push(prev);
                    rule.causes.push(poly.clone());
                }
                _ => panic!("Operator not allowed in a fat rule: '{}'.", op),
            }
            prev = poly;
        }
        rule
    }
}

#[derive(Clone, Default, Debug)]
pub struct NodeList {
    nodes: Vec<String>,
}

impl NodeList {
    pub(crate) fn from_node(node: String) -> Self {
        NodeList { nodes: vec![node] }
    }

    pub(crate) fn with_more(mut self, nodes: Vec<String>) -> Self {
        self.nodes.extend(nodes.into_iter());
        self
    }
}

#[derive(Clone, Default, Debug)]
pub struct Polynomial {
    monomials: BTreeSet<BTreeSet<String>>,
}

impl Polynomial {
    pub(crate) fn from_node(node: String) -> Self {
        Polynomial { monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node)))) }
    }

    /// Returns `self` multiplied by the product of `factors`.
    pub(crate) fn with_product_multiplied(mut self, mut factors: Vec<Self>) -> Self {
        self.multiply_assign(&mut factors);
        self
    }

    /// Returns `self` added to the product of `factors`.
    pub(crate) fn with_product_added(mut self, mut factors: Vec<Self>) -> Self {
        if let Some((head, tail)) = factors.split_first_mut() {
            head.multiply_assign(tail);
            self.add_assign(head);
        }
        self
    }

    fn multiply_assign(&mut self, factors: &mut [Self]) {
        for factor in factors {
            let lhs: Vec<_> = self.monomials.iter().cloned().collect();
            self.monomials.clear();

            for this_mono in lhs.iter() {
                for other_mono in factor.monomials.iter() {
                    let mut mono = this_mono.clone();
                    mono.extend(other_mono.iter().cloned());
                    self.monomials.insert(mono);
                }
            }
        }
    }

    fn add_assign(&mut self, other: &mut Self) {
        self.monomials.append(&mut other.monomials);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    Size(u64),
    Name(String),
}

impl Literal {
    pub(crate) fn from_digits(digits: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Literal::Size(u64::from_str(digits)?))
    }

    pub(crate) fn from_quoted_str(quoted: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Literal::Name(unquote(quoted)?))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    ThinTx,
    ThinRx,
    FatTx,
    FatRx,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinOp::Add => '+'.fmt(f),
            BinOp::ThinTx => "->".fmt(f),
            BinOp::ThinRx => "<-".fmt(f),
            BinOp::FatTx => "=>".fmt(f),
            BinOp::FatRx => "<=".fmt(f),
        }
    }
}
