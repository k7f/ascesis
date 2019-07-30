use std::{convert::TryFrom, iter::FromIterator};
use crate::Polynomial;

/// An alphabetically ordered and deduplicated list of `Node`s.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct NodeList {
    pub(crate) nodes: Vec<String>,
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
