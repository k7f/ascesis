use std::{cmp, collections::BTreeMap, convert::TryInto, error::Error};
use aces::{ContextHandle, node, monomial};
use crate::{Polynomial, Node, NodeList, Literal};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropSelector {
    Vis,
    SAT,
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct PropBlock {
    selector: Option<PropSelector>,
    fields:   BTreeMap<String, PropValue>,
}

impl PropBlock {
    pub fn new(key: String, value: PropValue) -> Self {
        let selector = Default::default();
        let mut fields = BTreeMap::new();
        fields.insert(key, value);

        PropBlock { selector, fields }
    }

    pub fn with_selector(mut self, selector: String) -> Self {
        match selector.as_str() {
            "vis" => self.selector = Some(PropSelector::Vis),
            "sat" => self.selector = Some(PropSelector::SAT),
            _ => {
                panic!() // FIXME
            }
        }

        self
    }

    #[inline]
    pub fn get_selector(&self) -> Option<PropSelector> {
        self.selector
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
            if let PropValue::Lit(Literal::Size(size)) = value {
                Some(*size)
            } else {
                None
            }
        })
    }

    pub fn get_name<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Lit(Literal::Name(name)) = value {
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PropValue {
    Lit(Literal),
    Identifier(String),
    Block(PropBlock),
}

impl From<Literal> for PropValue {
    fn from(lit: Literal) -> Self {
        PropValue::Lit(lit)
    }
}

impl From<String> for PropValue {
    fn from(identifier: String) -> Self {
        PropValue::Identifier(identifier)
    }
}

impl From<PropBlock> for PropValue {
    fn from(block: PropBlock) -> Self {
        PropValue::Block(block)
    }
}

/// A map from nodes to their capacities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CapacityBlock {
    capacities: BTreeMap<Node, u64>,
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

    pub(crate) fn compile(&self, ctx: &ContextHandle) -> Result<(), Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for (node, cap) in self.capacities.iter() {
            if let Some(cap) = node::Capacity::new_finite(*cap) {
                ctx.set_capacity(node.as_ref(), cap);
            } else {
                // FIXME omega
            }
        }

        Ok(())
    }
}

/// An alphabetically ordered and deduplicated list of multiplicities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MultiplicityBlock {
    multiplicities: Vec<Multiplicity>,
}

impl MultiplicityBlock {
    pub fn new_causes(
        size: Literal,
        post_nodes: Polynomial,
        pre_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let size = size.try_into()?;
        let post_nodes: NodeList = post_nodes.try_into()?;
        let pre_set: NodeList = pre_set.try_into()?;

        let multiplicities = post_nodes
            .nodes
            .into_iter()
            .map(|post_node| {
                Multiplicity::Rx(RxMultiplicity { size, post_node, pre_set: pre_set.clone() })
            })
            .collect();
        // No need to sort: `post_nodes` are already ordered and deduplicated.

        Ok(MultiplicityBlock { multiplicities })
    }

    pub fn new_effects(
        size: Literal,
        pre_nodes: Polynomial,
        post_set: Polynomial,
    ) -> Result<Self, Box<dyn Error>> {
        let size = size.try_into()?;
        let pre_nodes: NodeList = pre_nodes.try_into()?;
        let post_set: NodeList = post_set.try_into()?;

        let multiplicities = pre_nodes
            .nodes
            .into_iter()
            .map(|pre_node| {
                Multiplicity::Tx(TxMultiplicity { size, pre_node, post_set: post_set.clone() })
            })
            .collect();
        // No need to sort: `pre_nodes` are already ordered and deduplicated.

        Ok(MultiplicityBlock { multiplicities })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.multiplicities.append(&mut block.multiplicities);
        }

        self.multiplicities.sort();
        let len = self.multiplicities.partition_dedup().0.len();
        self.multiplicities.truncate(len);

        self
    }

    pub(crate) fn compile(&self, ctx: &ContextHandle) -> Result<(), Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for mul in self.multiplicities.iter() {
            match mul {
                Multiplicity::Rx(rx) => {
                    if let Some(weight) = monomial::Weight::new_finite(rx.size) {
                        let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                        ctx.set_weight(node::Face::Rx, rx.post_node.as_ref(), suit_names, weight);
                    } else {
                        // FIXME omega
                    }
                }
                Multiplicity::Tx(tx) => {
                    if let Some(weight) = monomial::Weight::new_finite(tx.size) {
                        let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                        ctx.set_weight(node::Face::Tx, tx.pre_node.as_ref(), suit_names, weight);
                    } else {
                        // FIXME omega
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Multiplicity {
    Rx(RxMultiplicity),
    Tx(TxMultiplicity),
}

impl cmp::Ord for Multiplicity {
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

impl cmp::PartialOrd for Multiplicity {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxMultiplicity {
    size:      u64,
    post_node: Node,
    pre_set:   NodeList,
}

impl cmp::Ord for RxMultiplicity {
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

impl cmp::PartialOrd for RxMultiplicity {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxMultiplicity {
    size:     u64,
    pre_node: Node,
    post_set: NodeList,
}

impl cmp::Ord for TxMultiplicity {
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

    pub(crate) fn compile(&self, ctx: &ContextHandle) -> Result<(), Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for inh in self.inhibitors.iter() {
            match inh {
                Inhibitor::Rx(rx) => {
                    let suit_names = rx.pre_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_inhibitor(node::Face::Rx, rx.post_node.as_ref(), suit_names);
                }
                Inhibitor::Tx(tx) => {
                    let suit_names = tx.post_set.nodes.iter().map(|n| n.as_ref());

                    ctx.set_inhibitor(node::Face::Tx, tx.pre_node.as_ref(), suit_names);
                }
            }
        }

        Ok(())
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
