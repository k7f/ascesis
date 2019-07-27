#![feature(slice_partition_dedup)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(
    #[allow(clippy::all)]
    pub ascesis_parser
);

lalrpop_mod!(
    #[allow(clippy::all)]
    pub bnf_parser
);

pub mod bnf;
pub mod grammar;
pub mod sentence;

use std::{
    cmp, fmt,
    collections::{BTreeSet, BTreeMap},
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    str::FromStr,
    error::Error,
};
use regex::Regex;
use enquote::unquote;
use crate::ascesis_parser::{
    CesFileBlockParser, ImmediateDefParser, CesInstanceParser, CapBlockParser, MulBlockParser,
    InhBlockParser, RexParser, ThinArrowRuleParser, FatArrowRuleParser, PolynomialParser,
};

pub type ParsingError = lalrpop_util::ParseError<usize, String, String>;
pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Clone, Debug)]
pub enum AscesisError {
    ParsingError(ParsingError),
    UnknownAxiom(String),
}

impl Error for AscesisError {
    fn description(&self) -> &str {
        use AscesisError::*;

        match self {
            ParsingError(_) => "ascesis parsing error",
            UnknownAxiom(_) => "unknown axiom",
        }
    }
}

impl fmt::Display for AscesisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AscesisError::*;

        match self {
            ParsingError(err) => {
                let message = format!("{}", err);
                let mut lines = message.lines();

                if let Some(line) = lines.next() {
                    writeln!(f, "{}", line)?;
                }

                for line in lines {
                    writeln!(f, "\t{}", line)?;
                }

                Ok(())
            }
            UnknownAxiom(symbol) => write!(f, "Unknown axiom '{}'", symbol),
        }
    }
}

impl From<ParsingError> for AscesisError {
    fn from(err: ParsingError) -> Self {
        AscesisError::ParsingError(err)
    }
}

#[derive(Clone, Debug)]
pub struct Axiom(String);

impl Axiom {
    pub fn from_known_symbol<S: AsRef<str>>(symbol: S) -> Option<Self> {
        let symbol = symbol.as_ref();

        match symbol {
            "CesFileBlock" | "ImmediateDef" | "CesInstance" | "CapBlock" | "MulBlock"
            | "InhBlock" | "Rex" | "ThinArrowRule" | "FatArrowRule" | "Polynomial" => {
                Some(Axiom(symbol.to_owned()))
            }
            _ => None,
        }
    }

    pub fn guess_from_spec<S: AsRef<str>>(spec: S) -> Self {
        lazy_static! {
            static ref IMM_RE: Regex = Regex::new(r"^ces\s+[[:alpha:]][[:word:]]*\s*\{").unwrap();
            static ref CAP_RE: Regex = Regex::new(r"^cap\s*\{").unwrap();
            static ref MUL_RE: Regex = Regex::new(r"^mul\s*\{").unwrap();
            static ref INH_RE: Regex = Regex::new(r"^inh\s*\{").unwrap();
            static ref TIN_RE: Regex = Regex::new(r"^[[:alpha:]][[:word:]]*\s*!\s*\(").unwrap();
            static ref IIN_RE: Regex =
                Regex::new(r"^[[:alpha:]][[:word:]]*\s*\(\s*\)\s*$").unwrap();
            static ref REX_RE: Regex = Regex::new(r"(\{|,|!|\(\s*\))").unwrap();
            static ref TAR_RE: Regex = Regex::new(r"(->|<-)").unwrap();
            static ref FAR_RE: Regex = Regex::new(r"(=>|<=)").unwrap();
        }

        let spec = spec.as_ref().trim();

        if IMM_RE.is_match(spec) {
            Axiom("ImmediateDef".to_owned())
        } else if CAP_RE.is_match(spec) {
            Axiom("CapBlock".to_owned())
        } else if MUL_RE.is_match(spec) {
            Axiom("MulBlock".to_owned())
        } else if INH_RE.is_match(spec) {
            Axiom("InhBlock".to_owned())
        } else if TIN_RE.is_match(spec) || IIN_RE.is_match(spec) {
            Axiom("CesInstance".to_owned())
        } else if REX_RE.is_match(spec) {
            Axiom("Rex".to_owned())
        } else if TAR_RE.is_match(spec) {
            Axiom("ThinArrowRule".to_owned())
        } else if FAR_RE.is_match(spec) {
            Axiom("FatArrowRule".to_owned())
        } else {
            // FIXME into tests: `a(b)` is a Polynomial, `a()`,
            // `a(b,)` are instantiations.
            Axiom("Polynomial".to_owned())
        }
    }

    pub fn symbol(&self) -> &str {
        self.0.as_str()
    }

    pub fn parse<S: AsRef<str>>(&self, spec: S) -> Result<Box<dyn FromSpec>, AscesisError> {
        macro_rules! from_spec_as {
            ($typ:ty, $spec:expr) => {{
                let object: $typ = $spec.parse()?;
                Ok(Box::new(object))
            }};
        }

        let spec = spec.as_ref();

        match self.0.as_str() {
            "CesFileBlock" => from_spec_as!(CesFileBlock, spec),
            "ImmediateDef" => from_spec_as!(ImmediateDef, spec),
            "CesInstance" => from_spec_as!(CesInstance, spec),
            "CapBlock" => from_spec_as!(CapacityBlock, spec),
            "MulBlock" => from_spec_as!(MultiplierBlock, spec),
            "InhBlock" => from_spec_as!(InhibitorBlock, spec),
            "Rex" => from_spec_as!(Rex, spec),
            "ThinArrowRule" => from_spec_as!(ThinArrowRule, spec),
            "FatArrowRule" => from_spec_as!(FatArrowRule, spec),
            "Polynomial" => from_spec_as!(Polynomial, spec),
            _ => Err(AscesisError::UnknownAxiom(self.0.clone())),
        }
    }
}

pub trait FromSpec: fmt::Debug {
    fn from_spec<S>(spec: S) -> ParsingResult<Self>
    where
        S: AsRef<str>,
        Self: Sized;
}

macro_rules! impl_from_str_for {
    ($nt:ty) => {
        impl FromStr for $nt {
            type Err = ParsingError;

            fn from_str(s: &str) -> ParsingResult<Self> {
                Self::from_spec(s)
            }
        }
    };
}

impl_from_str_for!(CesFileBlock);
impl_from_str_for!(ImmediateDef);
impl_from_str_for!(CesInstance);
impl_from_str_for!(CapacityBlock);
impl_from_str_for!(MultiplierBlock);
impl_from_str_for!(InhibitorBlock);
impl_from_str_for!(Rex);
impl_from_str_for!(ThinArrowRule);
impl_from_str_for!(FatArrowRule);
impl_from_str_for!(Polynomial);

#[derive(Debug)]
pub enum CesFileBlock {
    Imm(ImmediateDef),
    Cap(CapacityBlock),
    Mul(MultiplierBlock),
    Inh(InhibitorBlock),
}

impl From<ImmediateDef> for CesFileBlock {
    fn from(imm: ImmediateDef) -> Self {
        CesFileBlock::Imm(imm)
    }
}

impl From<CapacityBlock> for CesFileBlock {
    fn from(cap: CapacityBlock) -> Self {
        CesFileBlock::Cap(cap)
    }
}

impl From<MultiplierBlock> for CesFileBlock {
    fn from(mul: MultiplierBlock) -> Self {
        CesFileBlock::Mul(mul)
    }
}

impl From<InhibitorBlock> for CesFileBlock {
    fn from(inh: InhibitorBlock) -> Self {
        CesFileBlock::Inh(inh)
    }
}

impl FromSpec for CesFileBlock {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = CesFileBlockParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Debug)]
pub struct ImmediateDef {
    name: String,
    rex:  Rex,
}

impl ImmediateDef {
    pub fn new(name: String, rex: Rex) -> Self {
        ImmediateDef { name, rex }
    }
}

impl FromSpec for ImmediateDef {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = ImmediateDefParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Debug)]
pub struct CesInstance {
    name: String,
    args: Vec<String>,
}

impl CesInstance {
    pub(crate) fn new(name: String) -> Self {
        CesInstance { name, args: Vec::new() }
    }

    pub(crate) fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
}

impl FromSpec for CesInstance {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = CesInstanceParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

/// A map from nodes to their capacities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CapacityBlock {
    capacities: BTreeMap<String, u64>,
}

impl CapacityBlock {
    pub fn new(size: Literal, nodes: Polynomial) -> Result<Self, Box<dyn Error>> {
        let size = size.try_into()?;
        let nodes: NodeList = nodes.try_into()?;
        let mut capacities = BTreeMap::new();

        for node in nodes.nodes.into_iter() {
            capacities.insert(node, size);
        }

        Ok(CapacityBlock { capacities })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.capacities.append(&mut block.capacities);
        }
        self
    }
}

impl FromSpec for CapacityBlock {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = CapBlockParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

/// An alphabetically ordered and deduplicated list of `Multiplier`s.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MultiplierBlock {
    multipliers: Vec<Multiplier>,
}

impl MultiplierBlock {
    pub fn new_causes(
        size: Literal,
        post_nodes: Polynomial,
        pre_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let size = size.try_into()?;
        let post_nodes: NodeList = post_nodes.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;

        let multipliers = post_nodes
            .nodes
            .into_iter()
            .map(|post_node| {
                Multiplier::Rx(RxMultiplier { size, post_node, pre_set: pre_set.clone() })
            })
            .collect();
        // No need to sort: `post_nodes` are already ordered and deduplicated.

        Ok(MultiplierBlock { multipliers })
    }

    pub fn new_effects(
        size: Literal,
        pre_nodes: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let size = size.try_into()?;
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let post_set: NodeList = post_set.try_into()?;

        let multipliers = pre_nodes
            .nodes
            .into_iter()
            .map(|pre_node| {
                Multiplier::Tx(TxMultiplier { size, pre_node, post_set: post_set.clone() })
            })
            .collect();
        // No need to sort: `pre_nodes` are already ordered and deduplicated.

        Ok(MultiplierBlock { multipliers })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.multipliers.append(&mut block.multipliers);
        }

        self.multipliers.sort();
        let len = self.multipliers.partition_dedup().0.len();
        self.multipliers.truncate(len);

        self
    }
}

impl FromSpec for MultiplierBlock {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = MulBlockParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Multiplier {
    Rx(RxMultiplier),
    Tx(TxMultiplier),
}

impl cmp::Ord for Multiplier {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        use Multiplier::*;

        match self {
            Rx(s) => match other {
                Rx(o) => s.cmp(o),
                Tx(_) => cmp::Ordering::Less,
            },
            Tx(s) => match other {
                Rx(_) => cmp::Ordering::Greater,
                Tx(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for Multiplier {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxMultiplier {
    size:      u64,
    post_node: String,
    pre_set:   NodeList,
}

impl cmp::Ord for RxMultiplier {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_node.cmp(&other.post_node) {
            cmp::Ordering::Equal => match self.pre_set.cmp(&other.pre_set) {
                cmp::Ordering::Equal => self.size.cmp(&other.size),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxMultiplier {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxMultiplier {
    size:     u64,
    pre_node: String,
    post_set: NodeList,
}

impl cmp::Ord for TxMultiplier {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_node.cmp(&other.pre_node) {
            cmp::Ordering::Equal => match self.post_set.cmp(&other.post_set) {
                cmp::Ordering::Equal => self.size.cmp(&other.size),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxMultiplier {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// An alphabetically ordered and deduplicated list of `Inhibitor`s.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct InhibitorBlock {
    inhibitors: Vec<Inhibitor>,
}

impl InhibitorBlock {
    pub fn new_causes(post_nodes: Polynomial, pre_set: Polynomial) -> Result<Self, Box<dyn Error>> {
        let post_nodes: NodeList = post_nodes.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;

        let inhibitors = post_nodes
            .nodes
            .into_iter()
            .map(|post_node| Inhibitor::Rx(RxInhibitor { post_node, pre_set: pre_set.clone() }))
            .collect();
        // No need to sort: `post_nodes` are already ordered and deduplicated.

        Ok(InhibitorBlock { inhibitors })
    }

    pub fn new_effects(
        pre_nodes: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let post_set: NodeList = post_set.try_into()?;

        let inhibitors = pre_nodes
            .nodes
            .into_iter()
            .map(|pre_node| Inhibitor::Tx(TxInhibitor { pre_node, post_set: post_set.clone() }))
            .collect();
        // No need to sort: `pre_nodes` are already ordered and deduplicated.

        Ok(InhibitorBlock { inhibitors })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.inhibitors.append(&mut block.inhibitors);
        }

        self.inhibitors.sort();
        let len = self.inhibitors.partition_dedup().0.len();
        self.inhibitors.truncate(len);

        self
    }
}

impl FromSpec for InhibitorBlock {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = InhBlockParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Inhibitor {
    Rx(RxInhibitor),
    Tx(TxInhibitor),
}

impl cmp::Ord for Inhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        use Inhibitor::*;

        match self {
            Rx(s) => match other {
                Rx(o) => s.cmp(o),
                Tx(_) => cmp::Ordering::Less,
            },
            Tx(s) => match other {
                Rx(_) => cmp::Ordering::Greater,
                Tx(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for Inhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxInhibitor {
    post_node: String,
    pre_set:   NodeList,
}

impl cmp::Ord for RxInhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_node.cmp(&other.post_node) {
            cmp::Ordering::Equal => self.pre_set.cmp(&other.pre_set),
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxInhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxInhibitor {
    pre_node: String,
    post_set: NodeList,
}

impl cmp::Ord for TxInhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_node.cmp(&other.pre_node) {
            cmp::Ordering::Equal => self.post_set.cmp(&other.post_set),
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxInhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub type RexID = usize;

#[derive(Debug)]
pub struct Rex {
    root:  RexID,
    kinds: Vec<RexKind>,
}

impl Rex {
    pub(crate) fn with_more(self, rexlist: Vec<(Option<BinOp>, Rex)>) -> Self {
        let mut sum_pos = Vec::new();
        for (ndx, (op, _)) in rexlist.iter().enumerate() {
            if let Some(op) = op {
                if *op == BinOp::Add {
                    sum_pos.push(ndx);
                }
            }
        }

        self
    }
}

impl From<ThinArrowRule> for Rex {
    fn from(rule: ThinArrowRule) -> Self {
        Rex { root: 0, kinds: vec![RexKind::Thin(rule)] }
    }
}

impl From<FatArrowRule> for Rex {
    fn from(rule: FatArrowRule) -> Self {
        Rex { root: 0, kinds: vec![RexKind::Fat(rule)] }
    }
}

impl From<CesInstance> for Rex {
    fn from(instance: CesInstance) -> Self {
        Rex { root: 0, kinds: vec![RexKind::Instance(instance)] }
    }
}

impl FromSpec for Rex {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = RexParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Debug)]
pub enum RexKind {
    Thin(ThinArrowRule),
    Fat(FatArrowRule),
    Instance(CesInstance),
    Product(RexProduct),
    Sum(RexSum),
}

#[derive(Debug)]
pub struct RexProduct {
    ids: Vec<RexID>,
}

#[derive(Debug)]
pub struct RexSum {
    ids: Vec<RexID>,
}

#[derive(Default, Debug)]
pub struct ThinArrowRule {
    nodes:  NodeList,
    cause:  Polynomial,
    effect: Polynomial,
}

impl ThinArrowRule {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_nodes(mut self, nodes: Polynomial) -> Result<Self, Box<dyn Error>> {
        self.nodes = nodes.try_into()?;
        Ok(self)
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

impl FromSpec for ThinArrowRule {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = ThinArrowRuleParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct FatArrowRule {
    causes:  Vec<Polynomial>,
    effects: Vec<Polynomial>,
}

impl FatArrowRule {
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

impl FromSpec for FatArrowRule {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = FatArrowRuleParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

/// An alphabetically ordered and deduplicated list of `Node`s.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct NodeList {
    nodes: Vec<String>,
}

impl NodeList {
    pub fn with_more(mut self, nodes: Vec<String>) -> Self {
        self.nodes.extend(nodes.into_iter());
        self.nodes.sort();
        let len = self.nodes.partition_dedup().0.len();
        self.nodes.truncate(len);
        self
    }
}

impl From<String> for NodeList {
    fn from(node: String) -> Self {
        NodeList { nodes: vec![node] }
    }
}

impl From<Vec<String>> for NodeList {
    fn from(mut nodes: Vec<String>) -> Self {
        nodes.sort();
        let len = nodes.partition_dedup().0.len();
        nodes.truncate(len);
        NodeList { nodes }
    }
}

impl TryFrom<Polynomial> for NodeList {
    type Error = &'static str;

    fn try_from(poly: Polynomial) -> Result<Self, Self::Error> {
        if poly.is_flat {
            let mut monomials = poly.monomials.into_iter();

            if let Some(monomial) = monomials.next() {
                let nodes = Vec::from_iter(monomial.into_iter());
                // no need for sorting, unless `monomial` breaks the
                // invariants: 'is-ordered' and 'no-duplicates'...

                if monomials.next().is_none() {
                    Ok(NodeList { nodes })
                } else {
                    Err("Not a node list")
                }
            } else {
                Ok(Default::default())
            }
        } else {
            Err("Not a node list")
        }
    }
}

/// An alphabetically ordered and deduplicated list of monomials,
/// where each monomial is alphabetically ordered and deduplicated
/// list of `Node`s.
#[derive(Clone, Debug)]
pub struct Polynomial {
    monomials: BTreeSet<BTreeSet<String>>,

    // FIXME falsify on leading "+" or parens, even if still a mono
    is_flat: bool,
}

impl Polynomial {
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
            if !factor.is_flat {
                self.is_flat = false;
            }

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
        self.is_flat = false;
        self.monomials.append(&mut other.monomials);
    }
}

impl FromSpec for Polynomial {
    fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
        let spec = spec.as_ref();
        let mut errors = Vec::new();

        let result = PolynomialParser::new()
            .parse(&mut errors, spec)
            .map_err(|err| err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned()))?;

        Ok(result)
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Polynomial { monomials: BTreeSet::default(), is_flat: true }
    }
}

impl From<String> for Polynomial {
    fn from(node: String) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node)))),
            is_flat:   true,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
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

impl TryFrom<Literal> for u64 {
    type Error = &'static str;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        if let Literal::Size(size) = lit {
            Ok(size)
        } else {
            Err("Bad literal, not a size")
        }
    }
}

impl TryFrom<Literal> for String {
    type Error = &'static str;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        if let Literal::Name(name) = lit {
            Ok(name)
        } else {
            Err("Bad literal, not a string")
        }
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
