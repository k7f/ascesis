use std::{collections::BTreeMap, convert::TryInto, cmp, fmt, error::Error};
use aces::{ContextHandle, Compilable, Face, Capacity, Weight, sat};
use crate::{Polynomial, Node, NodeList, Literal, AscesisError, AscesisErrorKind};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PropSelector {
    AnonymousBlock,
    Vis,
    SAT,
    Invalid(String),
}

impl Default for PropSelector {
    #[inline]
    fn default() -> Self {
        PropSelector::AnonymousBlock
    }
}

impl fmt::Display for PropSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PropSelector::*;

        match self {
            AnonymousBlock => write!(f, "anonymous block"),
            Vis => write!(f, "Vis"),
            SAT => write!(f, "SAT"),
            Invalid(ref name) => write!(f, "{}", name),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PropValue {
    Literal(Literal),
    Identifier(String),
    SizeList(Vec<Literal>),
    IdentifierList(Vec<String>),
    Nodes(NodeList),
    Array(Vec<PropValue>),
    Block(PropBlock),
}

impl PropValue {
    pub(crate) fn into_array_with_more(self, more: Vec<Self>) -> Self {
        let mut values = match self {
            PropValue::Array(v) => v,
            _ => vec![self],
        };

        values.extend(more.into_iter());

        PropValue::Array(values)
    }

    pub(crate) fn new_name(lit: Literal) -> Result<Self, AscesisError> {
        if matches!(lit, Literal::Name(_)) {
            Ok(PropValue::Literal(lit))
        } else {
            Err(AscesisErrorKind::InvalidPropValueType("Name".into()).into())
        }
    }

    pub(crate) fn new_size_list(lits: Vec<Literal>) -> Result<Self, AscesisError> {
        if lits.iter().all(|lit| matches!(lit, Literal::Size(_))) {
            Ok(PropValue::SizeList(lits))
        } else {
            Err(AscesisErrorKind::InvalidPropValueType("Size".into()).into())
        }
    }

    pub(crate) fn new_node_list(names: Vec<String>) -> Result<Self, AscesisError> {
        Ok(PropValue::Nodes(names.into()))
    }
}

impl From<PropBlock> for PropValue {
    fn from(block: PropBlock) -> Self {
        PropValue::Block(block)
    }
}

impl From<Vec<PropValue>> for PropValue {
    fn from(vals: Vec<PropValue>) -> Self {
        PropValue::Array(vals)
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct PropBlock {
    selector: PropSelector,
    fields:   BTreeMap<String, PropValue>,
}

impl PropBlock {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_prop(mut self, key: String, value: PropValue) -> Self {
        self.fields.insert(key, value);

        self
    }

    pub(crate) fn with_selector(mut self, selector: String) -> Self {
        match selector.as_str() {
            "vis" => self.selector = PropSelector::Vis,
            "sat" => self.selector = PropSelector::SAT,
            _ => self.selector = PropSelector::Invalid(selector),
        }

        self
    }

    pub fn get_selector(&self) -> Result<PropSelector, AscesisError> {
        if let PropSelector::Invalid(ref selector) = self.selector {
            Err(AscesisErrorKind::InvalidPropSelector(selector.to_owned()).into())
        } else {
            Ok(self.selector.clone())
        }
    }

    pub fn verify_selector(&self, expected: PropSelector) -> Result<(), AscesisError> {
        let actual = self.get_selector()?;

        if actual == expected {
            Ok(())
        } else {
            Err(AscesisErrorKind::BlockSelectorMismatch(expected, actual).into())
        }
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.fields.append(&mut block.fields);
        }
        self
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&PropValue> {
        let key = key.as_ref();

        self.fields.get(key)
    }

    pub fn get_size<S: AsRef<str>>(&self, key: S) -> Option<u64> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Literal(Literal::Size(size)) = value {
                Some(*size)
            } else {
                None
            }
        })
    }

    pub fn get_name<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Literal(Literal::Name(name)) = value {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    pub fn get_identifier<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Identifier(identifier) = value {
                Some(identifier.as_str())
            } else {
                None
            }
        })
    }

    pub fn get_name_or_identifier<S: AsRef<str>>(
        &self,
        key: S,
    ) -> Result<Option<&str>, AscesisError> {
        let key = key.as_ref();

        if let Some(value) = self.fields.get(key) {
            match value {
                PropValue::Literal(Literal::Name(name)) => Ok(Some(name.as_str())),
                PropValue::Identifier(identifier) => Ok(Some(identifier.as_str())),
                _ => {
                    Err(AscesisErrorKind::InvalidPropType(self.selector.clone(), "key".to_owned())
                        .into())
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_nested_size<I, S>(&self, subblock_keys: I, value_key: S) -> Option<u64>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_size(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_size(value_key)
        }
    }

    pub fn get_nested_name<I, S>(&self, subblock_keys: I, value_key: S) -> Option<&str>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_name(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_name(value_key)
        }
    }

    pub fn get_nested_identifier<I, S>(&self, subblock_keys: I, value_key: S) -> Option<&str>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_identifier(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_identifier(value_key)
        }
    }

    pub fn get_sat_encoding(&self) -> Result<Option<sat::Encoding>, AscesisError> {
        self.verify_selector(PropSelector::SAT)?;

        if let Some(encoding) = self.get_name_or_identifier("encoding")? {
            match encoding {
                "port-link" => Ok(Some(sat::Encoding::PortLink)),
                "fork-join" => Ok(Some(sat::Encoding::ForkJoin)),
                _ => Err(AscesisErrorKind::InvalidPropValue(
                    PropSelector::SAT,
                    "encoding".to_owned(),
                    encoding.to_owned(),
                )
                .into()),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_sat_search(&self) -> Result<Option<sat::Search>, AscesisError> {
        self.verify_selector(PropSelector::SAT)?;

        if let Some(search) = self.get_name_or_identifier("search")? {
            match search {
                "min" => Ok(Some(sat::Search::MinSolutions)),
                "all" => Ok(Some(sat::Search::AllSolutions)),
                _ => Err(AscesisErrorKind::InvalidPropValue(
                    PropSelector::SAT,
                    "search".to_owned(),
                    search.to_owned(),
                )
                .into()),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_vis_title(&self) -> Result<Option<&str>, AscesisError> {
        self.verify_selector(PropSelector::Vis)?;

        Ok(self.get_name_or_identifier("title")?)
    }

    pub fn get_vis_labels(&self) -> Result<Option<&BTreeMap<String, PropValue>>, AscesisError> {
        self.verify_selector(PropSelector::Vis)?;

        if let Some(value) = self.fields.get("labels") {
            if let PropValue::Block(block) = value {
                Ok(Some(&block.fields))
            } else {
                Err(AscesisErrorKind::InvalidPropType(self.selector.clone(), "labels".to_owned())
                    .into())
            }
        } else {
            Ok(None)
        }
    }
}

impl Compilable for PropBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        match self.get_selector()? {
            PropSelector::Vis => {
                if let Some(title) = self.get_vis_title()? {
                    ctx.lock().unwrap().set_title(title);
                }

                if let Some(labels) = self.get_vis_labels()? {
                    for (node_name, node_label) in labels {
                        match node_label {
                            PropValue::Literal(Literal::Name(ref label))
                            | PropValue::Identifier(ref label) => {
                                let mut ctx = ctx.lock().unwrap();
                                let node_id = ctx.share_node_name(node_name);

                                ctx.set_label(node_id, label);
                            }
                            _ => {
                                return Err(AscesisError::from(AscesisErrorKind::InvalidPropType(
                                    PropSelector::Vis,
                                    "labels".to_owned(),
                                ))
                                .into())
                            }
                        }
                    }
                }
            }

            PropSelector::SAT => {
                if let Some(encoding) = self.get_sat_encoding()? {
                    info!("Using encoding '{:?}'", encoding);
                    ctx.lock().unwrap().set_encoding(encoding);
                }

                if let Some(search) = self.get_sat_search()? {
                    info!("Using '{:?}' search", search);
                    ctx.lock().unwrap().set_search(search);
                }
            }

            _ => unreachable!(),
        }

        Ok(true)
    }
}

/// A map from nodes to their capacities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CapacityBlock {
    capacities: BTreeMap<Node, Capacity>,
}

impl CapacityBlock {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_nodes(mut self, size: Literal, nodes: Polynomial) -> Result<Self, Box<dyn Error>> {
        let capacity = match size {
            Literal::Size(sz) => Capacity::finite(sz)
                .ok_or_else(|| AscesisError::from(AscesisErrorKind::SizeLiteralOverflow))?,
            Literal::Omega => Capacity::omega(),
            _ => return Err(AscesisError::from(AscesisErrorKind::ExpectedSizeLiteral).into()),
        };
        let nodes: NodeList = nodes.try_into()?;

        for node in nodes.nodes.into_iter() {
            self.capacities.insert(node, capacity);
        }

        Ok(self)
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.capacities.append(&mut block.capacities);
        }
        self
    }
}

impl Compilable for CapacityBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for (node, cap) in self.capacities.iter() {
            ctx.set_capacity_by_name(node.as_ref(), *cap);
        }

        Ok(true)
    }
}

/// An alphabetically ordered and deduplicated list of transfer
/// multiplicities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MultiplicityBlock {
    xfer_multiplicities: Vec<XferMultiplicity>,
}

impl MultiplicityBlock {
    pub fn new_causes(
        size: Literal,
        post_nodes: Polynomial,
        pre_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let weight = match size {
            Literal::Size(sz) => Weight::finite(sz)
                .ok_or_else(|| AscesisError::from(AscesisErrorKind::SizeLiteralOverflow))?,
            Literal::Omega => Weight::omega(),
            _ => return Err(AscesisError::from(AscesisErrorKind::ExpectedSizeLiteral).into()),
        };
        let post_nodes: NodeList = post_nodes.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;

        let xfer_multiplicities = post_nodes
            .nodes
            .into_iter()
            .map(|post_node| {
                XferMultiplicity::Rx(RxMultiplicity { weight, post_node, pre_set: pre_set.clone() })
            })
            .collect();
        // No need to sort: `post_nodes` are already ordered and deduplicated.

        Ok(MultiplicityBlock { xfer_multiplicities })
    }

    pub fn new_effects(
        size: Literal,
        pre_nodes: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let weight = match size {
            Literal::Size(sz) => Weight::finite(sz)
                .ok_or_else(|| AscesisError::from(AscesisErrorKind::SizeLiteralOverflow))?,
            Literal::Omega => Weight::omega(),
            _ => return Err(AscesisError::from(AscesisErrorKind::ExpectedSizeLiteral).into()),
        };
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let post_set: NodeList = post_set.try_into()?;

        let xfer_multiplicities = pre_nodes
            .nodes
            .into_iter()
            .map(|pre_node| {
                XferMultiplicity::Tx(TxMultiplicity {
                    weight,
                    pre_node,
                    post_set: post_set.clone(),
                })
            })
            .collect();
        // No need to sort: `pre_nodes` are already ordered and deduplicated.

        Ok(MultiplicityBlock { xfer_multiplicities })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.xfer_multiplicities.append(&mut block.xfer_multiplicities);
        }

        self.xfer_multiplicities.sort();
        let len = self.xfer_multiplicities.partition_dedup().0.len();
        self.xfer_multiplicities.truncate(len);

        self
    }
}

impl Compilable for MultiplicityBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for mul in self.xfer_multiplicities.iter() {
            match mul {
                XferMultiplicity::Rx(rx) => {
                    let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_weight_by_name(Face::Rx, rx.post_node.as_ref(), suit_names, rx.weight);
                }
                XferMultiplicity::Tx(tx) => {
                    let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_weight_by_name(Face::Tx, tx.pre_node.as_ref(), suit_names, tx.weight);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum XferMultiplicity {
    Rx(RxMultiplicity),
    Tx(TxMultiplicity),
}

impl cmp::Ord for XferMultiplicity {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Rx(s) => match other {
                Self::Rx(o) => s.cmp(o),
                Self::Tx(_) => cmp::Ordering::Less,
            },
            Self::Tx(s) => match other {
                Self::Rx(_) => cmp::Ordering::Greater,
                Self::Tx(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for XferMultiplicity {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RxMultiplicity {
    weight:    Weight,
    post_node: Node,
    pre_set:   NodeList,
}

impl cmp::Ord for RxMultiplicity {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_node.cmp(&other.post_node) {
            cmp::Ordering::Equal => match self.pre_set.cmp(&other.pre_set) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxMultiplicity {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TxMultiplicity {
    weight:   Weight,
    pre_node: Node,
    post_set: NodeList,
}

impl cmp::Ord for TxMultiplicity {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_node.cmp(&other.pre_node) {
            cmp::Ordering::Equal => match self.post_set.cmp(&other.post_set) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxMultiplicity {
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

impl Compilable for InhibitorBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for inh in self.inhibitors.iter() {
            match inh {
                Inhibitor::Rx(rx) => {
                    let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_inhibitor_by_name(Face::Rx, rx.post_node.as_ref(), suit_names);
                }
                Inhibitor::Tx(tx) => {
                    let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_inhibitor_by_name(Face::Tx, tx.pre_node.as_ref(), suit_names);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Inhibitor {
    Rx(RxInhibitor),
    Tx(TxInhibitor),
}

impl cmp::Ord for Inhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Rx(s) => match other {
                Self::Rx(o) => s.cmp(o),
                Self::Tx(_) => cmp::Ordering::Less,
            },
            Self::Tx(s) => match other {
                Self::Rx(_) => cmp::Ordering::Greater,
                Self::Tx(o) => s.cmp(o),
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
    post_node: Node,
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
    pre_node: Node,
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
