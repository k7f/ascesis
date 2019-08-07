use std::{cmp, collections::BTreeMap, convert::TryInto, error::Error};
use crate::{Polynomial, Node, NodeList, Literal};

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct VisBlock {
    fields: BTreeMap<String, VisValue>,
}

impl VisBlock {
    pub fn new(key: String, value: VisValue) -> Self {
        let mut fields = BTreeMap::new();
        fields.insert(key, value);

        VisBlock { fields }
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.fields.append(&mut block.fields);
        }
        self
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VisValue {
    Lit(Literal),
    Block(VisBlock),
}

impl From<Literal> for VisValue {
    fn from(lit: Literal) -> Self {
        VisValue::Lit(lit)
    }
}

impl From<VisBlock> for VisValue {
    fn from(block: VisBlock) -> Self {
        VisValue::Block(block)
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
    post_node: Node,
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
    pre_node: Node,
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
