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
    #[inline]
    pub(crate) fn new() -> Self {
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
                PropValue::Literal(Literal::Name(name)) => return Ok(Some(name.as_str())),
                PropValue::Identifier(identifier) => return Ok(Some(identifier.as_str())),
                PropValue::Nodes(nodes) => {
                    if nodes.nodes.len() == 1 {
                        return Ok(nodes.nodes.first().map(|n| n.as_ref()))
                    }
                }
                PropValue::IdentifierList(ids) => {
                    if ids.len() == 1 {
                        return Ok(ids.first().map(|i| i.as_str()))
                    }
                }
                _ => {}
            };

            Err(AscesisErrorKind::InvalidPropType(self.selector.clone(), "key".to_owned()).into())
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
pub struct CapacitiesBlock {
    capacities: BTreeMap<Node, Capacity>,
}

impl CapacitiesBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn with_nodes(mut self, size: Literal, nodes: Polynomial) -> Result<Self, AscesisError> {
        let capacity = match size {
            Literal::Size(sz) => Capacity::finite(sz)
                .ok_or_else(|| AscesisError::from(AscesisErrorKind::SizeLiteralOverflow))?,
            Literal::Omega => Capacity::omega(),
            _ => return Err(AscesisError::from(AscesisErrorKind::ExpectedSizeLiteral)),
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

impl Compilable for CapacitiesBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for (node, cap) in self.capacities.iter() {
            ctx.set_capacity_by_name(node.as_ref(), *cap);
        }

        Ok(true)
    }
}

/// A vector of unbounded capacity nodes.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct UnboundedBlock {
    nodes: Vec<Node>,
}

impl UnboundedBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn from_nodes(nodes: Polynomial) -> Result<Self, AscesisError> {
        let nodes: NodeList = nodes.try_into()?;

        Ok(UnboundedBlock { nodes: nodes.nodes })
    }
}

impl Compilable for UnboundedBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for node in self.nodes.iter() {
            ctx.set_capacity_by_name(node.as_ref(), Capacity::omega());
        }

        Ok(true)
    }
}

/// An alphabetically ordered and deduplicated list of transfer
/// multiplicities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct WeightsBlock {
    xfer_multiplicities: Vec<XferMultiplicity>,
}

impl WeightsBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_hyper_causes(
        size: Literal,
        post_nodes: Polynomial,
        pre_suit: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let post_nodes: NodeList = post_nodes.try_into()?;
        let pre_suit: NodeList = pre_suit.try_into()?;

        let xfer_multiplicities = post_nodes
            .nodes
            .into_iter()
            .map(|host_node| {
                XferMultiplicity::Rx(RxWeight {
                    weight,
                    host_node,
                    pre_suit: pre_suit.clone(),
                    post_set: None,
                })
            })
            .collect();
        // No need to sort: `post_nodes` are already ordered and deduplicated.

        Ok(WeightsBlock { xfer_multiplicities })
    }

    pub fn new_hyper_effects(
        size: Literal,
        pre_nodes: Polynomial,
        post_suit: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let post_suit: NodeList = post_suit.try_into()?;

        let xfer_multiplicities = pre_nodes
            .nodes
            .into_iter()
            .map(|host_node| {
                XferMultiplicity::Tx(TxWeight {
                    weight,
                    host_node,
                    pre_set: None,
                    post_suit: post_suit.clone(),
                })
            })
            .collect();
        // No need to sort: `pre_nodes` are already ordered and deduplicated.

        Ok(WeightsBlock { xfer_multiplicities })
    }

    pub fn new_flow_causes(
        size: Literal,
        host_node: Node,
        pre_set: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;
        let post_set: NodeList = post_set.try_into()?;
        let mult = XferMultiplicity::Rx(RxWeight {
            weight,
            host_node,
            pre_suit: pre_set,
            post_set: Some(post_set),
        });

        Ok(WeightsBlock { xfer_multiplicities: vec![mult] })
    }

    pub fn new_flow_effects(
        size: Literal,
        host_node: Node,
        pre_set: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;
        let post_set: NodeList = post_set.try_into()?;
        let mult = XferMultiplicity::Tx(TxWeight {
            weight,
            host_node,
            pre_set: Some(pre_set),
            post_suit: post_set,
        });

        Ok(WeightsBlock { xfer_multiplicities: vec![mult] })
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

impl Compilable for WeightsBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for weight in self.xfer_multiplicities.iter() {
            match weight {
                XferMultiplicity::Rx(rx) => {
                    let pre_names = rx.pre_suit.nodes.iter().map(|n| n.as_ref());

                    if let Some(ref post_set) = rx.post_set {
                        let post_names = post_set.nodes.iter().map(|n| n.as_ref());

                        ctx.set_flow_weight_by_names(
                            Face::Rx,
                            rx.host_node.as_ref(),
                            pre_names,
                            post_names,
                            rx.weight,
                        )?;
                    } else {
                        ctx.set_hyper_weight_by_names(
                            Face::Rx,
                            rx.host_node.as_ref(),
                            pre_names,
                            rx.weight,
                        );
                    }
                }
                XferMultiplicity::Tx(tx) => {
                    let post_names = tx.post_suit.nodes.iter().map(|n| n.as_ref());

                    if let Some(ref pre_set) = tx.pre_set {
                        let pre_names = pre_set.nodes.iter().map(|n| n.as_ref());

                        ctx.set_flow_weight_by_names(
                            Face::Tx,
                            tx.host_node.as_ref(),
                            pre_names,
                            post_names,
                            tx.weight,
                        )?;
                    } else {
                        ctx.set_hyper_weight_by_names(
                            Face::Tx,
                            tx.host_node.as_ref(),
                            post_names,
                            tx.weight,
                        );
                    }
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum XferMultiplicity {
    Rx(RxWeight),
    Tx(TxWeight),
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
struct RxWeight {
    weight:    Weight,
    host_node: Node,
    pre_suit:  NodeList, // this is the entire pre_set if post_set is specified
    post_set:  Option<NodeList>,
}

impl cmp::Ord for RxWeight {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.host_node.cmp(&other.host_node) {
            cmp::Ordering::Equal => match self.pre_suit.cmp(&other.pre_suit) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxWeight {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TxWeight {
    weight:    Weight,
    host_node: Node,
    pre_set:   Option<NodeList>,
    post_suit: NodeList, // this is the entire post_set if pre_set is specified
}

impl cmp::Ord for TxWeight {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.host_node.cmp(&other.host_node) {
            cmp::Ordering::Equal => match self.post_suit.cmp(&other.post_suit) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxWeight {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// An alphabetically ordered and deduplicated list of `Inhibitor`s.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct InhibitorsBlock {
    inhibitors: Vec<Inhibitor>,
}

impl InhibitorsBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_causes(post_nodes: Polynomial, pre_poly: Polynomial) -> Result<Self, AscesisError> {
        let post_nodes: NodeList = post_nodes.try_into()?;
        let mut inhibitors = Vec::new();

        // `post_nodes` are already ordered and deduplicated
        for post_node in post_nodes.nodes {
            // monomials are already ordered and deduplicated
            for mono in pre_poly.monomials.iter() {
                let post_node = post_node.clone();
                let pre_set = mono.clone().into();

                inhibitors.push(Inhibitor::Rx(RxInhibitor { post_node, pre_set }));
            }
        }

        Ok(InhibitorsBlock { inhibitors })
    }

    pub fn new_effects(pre_nodes: Polynomial, post_poly: Polynomial) -> Result<Self, AscesisError> {
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let mut inhibitors = Vec::new();

        // `pre_nodes` are already ordered and deduplicated
        for pre_node in pre_nodes.nodes {
            // monomials are already ordered and deduplicated
            for mono in post_poly.monomials.iter() {
                let pre_node = pre_node.clone();
                let post_set = mono.clone().into();

                inhibitors.push(Inhibitor::Tx(TxInhibitor { pre_node, post_set }));
            }
        }

        Ok(InhibitorsBlock { inhibitors })
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

impl Compilable for InhibitorsBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for inhibitor in self.inhibitors.iter() {
            match inhibitor {
                Inhibitor::Rx(rx) => {
                    let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_hyper_inhibitor_by_names(Face::Rx, rx.post_node.as_ref(), suit_names);
                }
                Inhibitor::Tx(tx) => {
                    let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_hyper_inhibitor_by_names(Face::Tx, tx.pre_node.as_ref(), suit_names);
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

/// An alphabetically ordered and deduplicated list of `Weightless` splits.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct WeightlessBlock {
    pub(crate) face:   Option<Face>,
    pub(crate) splits: Vec<Weightless>,
}

impl WeightlessBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_causes(post_nodes: Polynomial, pre_poly: Polynomial) -> Result<Self, AscesisError> {
        let face = Some(Face::Rx);
        let post_nodes: NodeList = post_nodes.try_into()?;
        let mut splits = Vec::new();

        // `post_nodes` are already ordered and deduplicated
        for post_node in post_nodes.nodes {
            // monomials are already ordered and deduplicated
            for mono in pre_poly.monomials.iter() {
                let post_node = post_node.clone();
                let pre_set = mono.clone().into();

                splits.push(Weightless::Drop(RxWeightless { post_node, pre_set }));
            }
        }

        Ok(WeightlessBlock { face, splits })
    }

    pub fn new_effects(pre_nodes: Polynomial, post_poly: Polynomial) -> Result<Self, AscesisError> {
        let face = Some(Face::Tx);
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let mut splits = Vec::new();

        // `pre_nodes` are already ordered and deduplicated
        for pre_node in pre_nodes.nodes {
            // monomials are already ordered and deduplicated
            for mono in post_poly.monomials.iter() {
                let pre_node = pre_node.clone();
                let post_set = mono.clone().into();

                splits.push(Weightless::Activate(TxWeightless { pre_node, post_set }));
            }
        }

        Ok(WeightlessBlock { face, splits })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            if self.face.is_some() && block.face != self.face {
                self.face = None;
            }

            self.splits.append(&mut block.splits);
        }

        self.splits.sort();
        let len = self.splits.partition_dedup().0.len();
        self.splits.truncate(len);

        self
    }

    #[inline]
    pub fn get_face(&self) -> Option<Face> {
        self.face
    }
}

impl From<WeightlessBlock> for WeightsBlock {
    fn from(block: WeightlessBlock) -> Self {
        let mut more_weights = Vec::new();

        for split in block.splits {
            // FIXME unwraps
            match split {
                Weightless::Activate(activate) => {
                    more_weights.push(
                        WeightsBlock::new_hyper_effects(
                            Literal::Size(0),
                            activate.pre_node.into(),
                            activate.post_set.into(),
                        )
                        .unwrap(),
                    );
                }
                Weightless::Drop(drop) => {
                    more_weights.push(
                        WeightsBlock::new_hyper_causes(
                            Literal::Size(0),
                            drop.post_node.into(),
                            drop.pre_set.into(),
                        )
                        .unwrap(),
                    );
                }
            }
        }

        WeightsBlock::new().with_more(more_weights)
    }
}

impl Compilable for WeightlessBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for holder in self.splits.iter() {
            match holder {
                Weightless::Activate(tx) => {
                    let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_hyper_holder_by_names(Face::Tx, tx.pre_node.as_ref(), suit_names);
                }
                Weightless::Drop(rx) => {
                    let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_hyper_holder_by_names(Face::Rx, rx.post_node.as_ref(), suit_names);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Weightless {
    Activate(TxWeightless),
    Drop(RxWeightless),
}

impl cmp::Ord for Weightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Activate(s) => match other {
                Self::Activate(o) => s.cmp(o),
                Self::Drop(_) => cmp::Ordering::Greater,
            },
            Self::Drop(s) => match other {
                Self::Activate(_) => cmp::Ordering::Less,
                Self::Drop(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for Weightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxWeightless {
    pre_node: Node,
    post_set: NodeList,
}

impl cmp::Ord for TxWeightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_node.cmp(&other.pre_node) {
            cmp::Ordering::Equal => self.post_set.cmp(&other.post_set),
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxWeightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxWeightless {
    post_node: Node,
    pre_set:   NodeList,
}

impl cmp::Ord for RxWeightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_node.cmp(&other.post_node) {
            cmp::Ordering::Equal => self.pre_set.cmp(&other.pre_set),
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxWeightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
